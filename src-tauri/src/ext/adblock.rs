use adblock::engine::Engine;
use adblock::lists::{FilterSet, ParseOptions};
use adblock::request::Request;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

const ADGUARD_BASE_URL: &str = "https://filters.adtidy.org/windows/filters/2.txt";
const ADGUARD_TRACKING_URL: &str = "https://filters.adtidy.org/windows/filters/3.txt";

const FILTERS_DIR: &str = "adblock-filters";
const ENGINE_CACHE: &str = "engine_cache.bin";

const BUNDLED_RULES: &str = include_str!("./adblock_bundled.txt");

pub struct AdblockManager {
    engine: Mutex<Engine>,
}

unsafe impl Send for AdblockManager {}
unsafe impl Sync for AdblockManager {}

impl AdblockManager {
    pub fn new_with_bundled() -> Self {
        let mut filter_set = FilterSet::new(false);
        filter_set.add_filter_list(BUNDLED_RULES, ParseOptions::default());
        let engine = Engine::from_filter_set(filter_set, true);
        Self {
            engine: Mutex::new(engine),
        }
    }

    pub async fn load_filters(&self, app: &AppHandle) {
        let data_dir = match app.path().app_data_dir() {
            Ok(d) => d,
            Err(_) => return,
        };

        let cache_path = data_dir.join(ENGINE_CACHE);

        if let Ok(data) = std::fs::read(&cache_path) {
            let mut engine = Engine::default();
            if engine.deserialize(&data).is_ok() {
                let mut current = self.engine.lock().unwrap();
                *current = engine;
                return;
            }
        }

        let filters_dir = data_dir.join(FILTERS_DIR);
        let _ = std::fs::create_dir_all(&filters_dir);

        let base_result = download_list(ADGUARD_BASE_URL).await;
        let tracking_result = download_list(ADGUARD_TRACKING_URL).await;

        if let Ok(ref content) = base_result {
            let _ = std::fs::write(filters_dir.join("adguard_base.txt"), content);
        }
        if let Ok(ref content) = tracking_result {
            let _ = std::fs::write(filters_dir.join("adguard_tracking.txt"), content);
        }

        let mut filter_set = FilterSet::new(false);
        filter_set.add_filter_list(BUNDLED_RULES, ParseOptions::default());
        if let Ok(ref content) = base_result {
            filter_set.add_filter_list(content, ParseOptions::default());
        }
        if let Ok(ref content) = tracking_result {
            filter_set.add_filter_list(content, ParseOptions::default());
        }

        let engine = Engine::from_filter_set(filter_set, true);
        let serialized = engine.serialize();
        let _ = std::fs::write(&cache_path, &serialized);

        let mut current = self.engine.lock().unwrap();
        *current = engine;
    }

    pub fn is_blocked(&self, url: &str, source_url: &str, request_type: &str) -> bool {
        let request = match Request::new(url, source_url, request_type) {
            Ok(r) => r,
            Err(_) => return false,
        };
        let engine = self.engine.lock().unwrap();
        let result = engine.check_network_request(&request);
        result.matched
    }

    pub fn get_cosmetic_filters(&self, url: &str) -> CosmeticFilters {
        let engine = self.engine.lock().unwrap();
        let resources = engine.url_cosmetic_resources(url);
        CosmeticFilters {
            hide_selectors: resources.hide_selectors.into_iter().collect(),
            injected_script: resources.injected_script.clone(),
            generichide: resources.generichide,
        }
    }
}

#[derive(serde::Serialize)]
pub struct CosmeticFilters {
    pub hide_selectors: Vec<String>,
    pub injected_script: String,
    pub generichide: bool,
}

async fn download_list(url: &str) -> Result<String, reqwest::Error> {
    reqwest::get(url).await?.text().await
}

#[tauri::command]
pub fn check_url_blocked(
    state: tauri::State<'_, AdblockManager>,
    url: String,
    source_url: String,
    request_type: String,
) -> Result<bool, String> {
    Ok(state.is_blocked(&url, &source_url, &request_type))
}

#[tauri::command]
pub fn get_page_cosmetic_filters(
    state: tauri::State<'_, AdblockManager>,
    url: String,
) -> Result<CosmeticFilters, String> {
    Ok(state.get_cosmetic_filters(&url))
}

pub fn adblock_script() -> String {
    r#"(function () {
    'use strict';

    const SITE_ORIGIN = location.origin;
    var cache = new Map();

    function cacheKey(url, type) { return url + '|' + type; }

    function checkViaIpc(url, type) {
        var key = cacheKey(url, type);
        if (cache.has(key)) return Promise.resolve(cache.get(key));
        return window.__TAURI_INTERNALS__.invoke('check_url_blocked', {
            url: url, sourceUrl: SITE_ORIGIN, requestType: type
        }).then(function (b) {
            cache.set(key, b);
            if (cache.size > 50000) cache.clear();
            return b;
        }).catch(function () { return false; });
    }

    function syncCheck(url, type) {
        var key = cacheKey(url, type);
        return cache.has(key) ? cache.get(key) : null;
    }

    function rm(el) {
        if (el && el.parentNode) try { el.parentNode.removeChild(el); } catch (_) {}
    }

    var typeMap = { script:'script', iframe:'subdocument', img:'image', video:'media', source:'media', embed:'object', object:'object' };
    function rtype(tag) { return typeMap[tag] || 'other'; }

    function checkEl(el) {
        if (!el || el.nodeType !== 1) return;
        var tag = el.nodeName.toLowerCase();
        if (tag === 'script' || tag === 'iframe' || tag === 'img' || tag === 'video' || tag === 'source' || tag === 'embed' || tag === 'object') {
            var src = el.src || el.getAttribute('src') || '';
            if (src && src.length > 4) {
                var cached = syncCheck(src, rtype(tag));
                if (cached === true) { rm(el); return; }
                if (cached === null) {
                    (function(e, s, t) {
                        checkViaIpc(s, t).then(function (b) { if (b) rm(e); });
                    })(el, src, rtype(tag));
                }
            }
        }
    }

    var _fetch = window.fetch.bind(window);
    window.fetch = function (resource, init) {
        var url;
        if (typeof resource === 'string') url = resource;
        else if (resource instanceof Request) url = resource.url;
        else return _fetch(resource, init);
        return checkViaIpc(url, 'xmlhttprequest').then(function (b) {
            return b ? Promise.reject(new TypeError('Blocked')) : _fetch(resource, init);
        });
    };

    var _xhrOpen = XMLHttpRequest.prototype.open;
    XMLHttpRequest.prototype.open = function (method, url) {
        this._abUrl = typeof url === 'string' ? url : '';
        return _xhrOpen.apply(this, arguments);
    };
    var _xhrSend = XMLHttpRequest.prototype.send;
    XMLHttpRequest.prototype.send = function (body) {
        if (this._abUrl) {
            var cached = syncCheck(this._abUrl, 'xmlhttprequest');
            if (cached === true) { try { this.abort(); } catch (_) {} return; }
            if (cached === null) {
                var self = this;
                checkViaIpc(this._abUrl, 'xmlhttprequest').then(function (b) { if (b) try { self.abort(); } catch (_) {} });
            }
        }
        return _xhrSend.call(this, body);
    };

    var _createEl = document.createElement.bind(document);
    var SRC_TAGS = { img:1, iframe:1, video:1, source:1, embed:1, object:1 };
    document.createElement = function (tag) {
        var el = _createEl.apply(document, arguments);
        if (typeof tag !== 'string') return el;
        var lower = tag.toLowerCase();
        if (!SRC_TAGS[lower]) return el;
        var proto = Object.getPrototypeOf(el);
        var desc = Object.getOwnPropertyDescriptor(proto, 'src');
        if (desc && desc.set) {
            var nativeSet = desc.set;
            Object.defineProperty(el, 'src', {
                get: function () { return desc.get ? desc.get.call(this) : ''; },
                set: function (val) {
                    var str = String(val);
                    var cached = syncCheck(str, rtype(lower));
                    if (cached === true) return;
                    if (cached === null) {
                        checkViaIpc(str, rtype(lower)).then(function (b) { if (!b) nativeSet.call(el, val); });
                        return;
                    }
                    nativeSet.call(el, val);
                },
                configurable: true
            });
        }
        return el;
    };

    var observer = null;
    function startObs() {
        if (observer) return;
        observer = new MutationObserver(function (mutations) {
            for (var m = 0; m < mutations.length; m++) {
                var added = mutations[m].addedNodes;
                if (added) {
                    for (var i = 0; i < added.length; i++) checkEl(added[i]);
                }
            }
        });
        observer.observe(document.documentElement, { childList: true, subtree: true });
    }
    if (document.body) startObs(); else document.addEventListener('DOMContentLoaded', startObs, { once: true });

    var style = document.createElement('style');
    style.id = '__ab__';
    style.textContent = 'ins.adsbygoogle,.adsbygoogle,[id^="ad-"],[class*="ad-"],[id*="google_ads"]{display:none!important}';
    (document.head || document.documentElement).appendChild(style);

    function loadCosmetic() {
        window.__TAURI_INTERNALS__.invoke('get_page_cosmetic_filters', { url: location.href })
            .then(function (f) {
                if (f.injected_script) {
                    try {
                        var s = document.createElement('script');
                        s.id = '__ab_scriptlets__';
                        s.textContent = f.injected_script;
                        document.head.appendChild(s);
                    } catch (_) {}
                }
                if (f.hide_selectors && f.hide_selectors.length) {
                    var prev = document.getElementById('__ab_cosmetic__');
                    if (prev) prev.parentNode.removeChild(prev);
                    var st = document.createElement('style');
                    st.id = '__ab_cosmetic__';
                    st.textContent = f.hide_selectors.join(',\n') + '{display:none!important}';
                    document.head.appendChild(st);
                }
            }).catch(function () {});
    }
    if (document.readyState === 'complete') setTimeout(loadCosmetic, 100);
    else window.addEventListener('load', function () { setTimeout(loadCosmetic, 100); }, { once: true });
}());
"#.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_contains_invoke() {
        let script = adblock_script();
        assert!(script.contains("check_url_blocked"));
        assert!(script.contains("get_page_cosmetic_filters"));
    }

    #[test]
    fn bundled_file_exists() {
        assert!(!BUNDLED_RULES.is_empty());
        assert!(BUNDLED_RULES.contains("doubleclick"));
    }
}
