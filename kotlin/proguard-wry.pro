# THIS FILE IS AUTO-GENERATED. DO NOT MODIFY!!

# Copyright 2020-2023 Tauri Programme within The Commons Conservancy
# SPDX-License-Identifier: Apache-2.0
# SPDX-License-Identifier: MIT

-keep class dev.dioxus.main.* {
  native <methods>;
}

-keep class dev.dioxus.main.WryActivity {
  public <init>(...);

  void setWebView(dev.dioxus.main.RustWebView);
  java.lang.Class getAppClass(...);
  java.lang.String getVersion();
}

-keep class dev.dioxus.main.Ipc {
  public <init>(...);

  @android.webkit.JavascriptInterface public <methods>;
}

-keep class dev.dioxus.main.RustWebView {
  public <init>(...);

  void loadUrlMainThread(...);
  void loadHTMLMainThread(...);
  void evalScript(...);
}

-keep class dev.dioxus.main.RustWebChromeClient,dev.dioxus.main.RustWebViewClient {
  public <init>(...);
}
