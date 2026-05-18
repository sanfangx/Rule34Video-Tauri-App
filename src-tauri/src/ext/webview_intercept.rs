#[cfg(windows)]
mod imp {
    use tauri::{AppHandle, Manager};
    use webview2_com::{Microsoft::Web::WebView2::Win32::*, WebResourceRequestedEventHandler};
    use windows::Win32::System::Com::IStream;
    use windows_core::{HSTRING, PWSTR};

    pub fn setup(app: &AppHandle) {
        let app_handle = app.clone();
        let window = match app_handle.get_webview_window("main") {
            Some(w) => w,
            None => return,
        };

        let _ = window.as_ref().with_webview(move |pw| {
            let controller = pw.controller();
            let env = pw.environment();

            let core = match unsafe { controller.CoreWebView2() } {
                Ok(c) => c,
                Err(_) => return,
            };
            let handler = WebResourceRequestedEventHandler::create(Box::new(move |_sender, args| {
                let args = match args {
                    Some(a) => a,
                    None => return Ok(()),
                };

                let request = match unsafe { args.Request() } {
                    Ok(r) => r,
                    Err(_) => return Ok(()),
                };

                let mut uri_ptr = PWSTR::null();
                if unsafe { request.Uri(&mut uri_ptr) }.is_err() {
                    return Ok(());
                }

                let uri_str = if !uri_ptr.is_null() {
                    unsafe {
                        let len = (0..)
                            .take_while(|&i| *uri_ptr.0.add(i) != 0)
                            .count();
                        String::from_utf16_lossy(std::slice::from_raw_parts(uri_ptr.0, len))
                    }
                } else {
                    return Ok(());
                };

                let manager = app_handle.state::<crate::ext::adblock::AdblockManager>();
                if manager.is_blocked(&uri_str, "", "other") {
                    if let Ok(response) = unsafe {
                        env.CreateWebResourceResponse(
                            None::<&IStream>,
                            204,
                            &HSTRING::new(),
                            &HSTRING::new(),
                        )
                    } {
                        let _ = unsafe { args.SetResponse(&response) };
                    }
                }

                Ok(())
            }));

            unsafe {
                let _ = core.AddWebResourceRequestedFilter(
                    &HSTRING::from("*"),
                    COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL,
                );
                let mut token: i64 = 0;
                let _ = core.add_WebResourceRequested(&handler, &mut token);
            }
        });
    }
}

#[cfg(not(windows))]
mod imp {
    use tauri::AppHandle;
    pub fn setup(_app: &AppHandle) {}
}

pub use imp::setup;
