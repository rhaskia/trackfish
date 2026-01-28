# TrackFish
![](https://raw.githubusercontent.com/rhaskia/trackfish/refs/heads/main/image.png)
TrackFish is a music player made for offline usage without having to sacrifice features typically only associated with streaming services - the ability to have similar songs to continue playing after one ends, called autoplay or a radio in many apps. TrackFish also seeks to be a generally comprehensive music player with features such as playlists, autoplaylists, and comprehensive queue management. Planned features include recommendations, auto-playlist downloading, and tagging support.

# Compiling
On desktop, you will need the dioxus-cli, and to install the following packages:
Linux: WebkitGtk, xdotool
Windows: WebView2 (packaged with Edge),
and aubio on all platforms

Android is supported, and should compile fine if the Android SDK is set up properly. iOS probably does not compile due to a lack of native bindings. 

# To Do:
 - [x] Audio playing, skipping, etc
 - [x] Working track view
 - [x] Proper Album & Artist Views
    - [x] More view information (time, artists, etc)
    - [x] Track settings (play, play after, start radio)
 - [x] Shuffle/Unshuffle
 - [x] Custom Music Folder
 - [x] All tracks search
 - [ ] Search 
    - [x] Search for albums/artists/genres
    - [x] Search View
    - [ ] Better search algorithm
 - [ ] Media notifications/control
    - [x] Android
    - [ ] Desktop
 - [x] Playlists 
    - [x] Creation
    - [x] Playing as queue
    - [x] Saving
    - [x] Adding tracks
    - [x] Deletion
    - [x] Removing tracks
 - [ ] Settings
    - [x] Settings View
    - [x] Radio settings (weights, temperature, etc)
    - [ ] Audio settings (volume, fade, etc)
    - [ ] Library settings
    - [ ] View Settings
 - [x] Radio playing system
 - [x] More Weighting
    - [x] Spectral
    - [x] Chroma
    - [x] MFCCs
    - [x] Zero Crossing Rate
    - [x] Energy
    - [x] BPM/Tempo
 - [ ] Queue Management
    - [x] Switch queues
    - [x] Select song in queue
    - [x] Drag and drop
    - [x] Add track list to queue
    - [ ] Locking queues to stop them being recached on new load/scan?
    - [ ] End of queue options - stop, next, repeat, reshuffle etc
    - [ ] Sorting features
 - [x] Auto Playlists
    - [x] Sort by metadata
    - [ ] Automatic re-caching onload or on any change?
    - [ ] Sort by audio features
 - [ ] Theming (loading of custom css)
 - [ ] Tagging/Collection Management
    - [x] Tag Editor
    - [ ] Auto Tagging (MusicBrainz?)
    - [ ] Albums with missing tracks
    - [x] Song Options
    - [ ] Multi-select song options
 - [ ] Music Exploration?
    - [ ] Auto Downloads (yt-dlp?)
    - [ ] Last FM-based reccomendations


curl.exe ^"https://www.last.fm/charts/partial/trending-tracks?ajax=1^" ^
  --compressed ^
  -H ^"User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:147.0) Gecko/20100101 Firefox/147.0^" ^
  -H ^"Accept: */*^" ^
  -H ^"Accept-Language: en-US,en;q=0.9^" ^
  -H ^"Accept-Encoding: gzip, deflate, br, zstd^" ^
  -H ^"X-NewRelic-ID: UwYPV15QGwYFXFlXDgU=^" ^
  -H ^"X-Requested-With: XMLHttpRequest^" ^
  -H ^"Sec-GPC: 1^" ^
  -H ^"Connection: keep-alive^" ^
  -H ^"Referer: https://www.last.fm/user/Rhaskia/playlists/14463484^" ^
  -H ^"Cookie: sessionid=.eJyVVsmOq0gW_Zdct_Mx2vCkt8DGYEgGM0NsLAKwmcFmplT_3kFmlrpKpZKqF5aAO517457j-O3tFg59ehu65HVLwy59-_k2YseQaaLMdGnKS9pIPibwzLCBN195vX0d3v7z1iVdlzX1LYuRf4KRMCHDaMfQ93hHHWJ8x8QxucP2BMUkIRXRRIhi_lToM-znz5-OdTYlHj3gGIMxFMWSf_WDYVQk9eZc3qv3KYHv0zS9h23bvW9O79_2d-fVHb9dUXw7vDoUotrRrNsSpvMqofMFoRm_fm3m7sucLHIKxSjTM9lyVgnXMqmTqr4FJ2kvlbIfFOzTsGT2HTm2Ealujk18MadobUaF1MqI1FpIUAOoygV4choRzgAIdgTEXCqVNkKLbRVvTiPSrKCFp7FvNlI2ZYHvFlLeZKGFd4GvYaDCevQNC08bAAGPxfQei2UPLGmPesC1vFhVu6B1a8piH9Ut2S-QuTppNoepvESruTOrvIqhfkmN52Ytl1bdPlPItuh2MW91I9LNtrrqyuEqb0xqXiyafZ7U1UDxEcpzJtRcpVX7gWwSjeIz5SS3gW9ken4mNTug1fWMvhcIp0tFp29864Pa4jZ8wMPT0Ju2WQ2BR48RGY9QBGNMzD3wtMYQ4z7wyu5rPlu_7ue7T4Iyqsoq9LbZtFhUuyWqXYW-XAaVsG6z2M4BiO6AZoMhvxX5Hf6IR89QqugUVuwaE2wHKpaEIl4GRMoB4hPTEBBsL5U4i_KuoRcPPlEOPnlcIAm2vMs2m0gU1g1zVBlb7xjwUM7a2PrBo1r9Pid8iiq2UDxzjC9nXKpxtCRteFY4Roryo3NnBcqcn0_Ot05WU35ILrOoN55oWladJZ1TfLEmuqfecNeYMLT-sfOnyaZzM7bSC9Zi7ujnuFn4hn2oZD3DetO9X5ZL73fQukRC2ex9uOc-IP9krQZc4uK2LN4STG2WQjPhBXuiioUbo_2txcrJk3zysvC-Yux6UzHpaMASk8oFhVg5LH09XJgGkvEiCfG-7qYhsxYJyJx5kvyD8STEhCEhYY5CpoU3updPbkoZrxQSmCCAVREb3jAztjsmSqPolEWOzd7KvLSnvI_YyKu8B7x27dxXsTvQ_o4qZ-izH-U9merXDhRmJ2i87BBS-tifort_q1i9ECVSqhIsuKiPja79v6Br5VZqblAgNxaANjnwnAnkxcaGKfACOrDLFG03YlFcglwu1NWtwNd2Y4nPbTk_XNzI7sY_kj0g3QX45hgS7qBUaKmJuAMWnaNBjP9E5C9oRQ_RUsUi6BVvI_djK_spBj5pttBz_k6u2l2BL99Db1s8aVV5Z0KkRkSWcNXmVo1HxF0Rge1gUddi1pENkZlWN0LVGh5k_784hJ6AIQLM2uoQuq0i4SxohLNMLtyf8Dn4N_m_SddksBIGSGpoR-YKknEHPTcPz8caekILT5_z2fotP98FlkB9tUgwy2028UXGwSYa_xPQTK87hMUsYW22iOhltHwS_ysexXjLvxEWOb6jvBFR1hCJJfpthCW2vNE2m_qIRBlhJuVi6z1GooJybv0jgZXX73Pqo4s8ghNeQNKcNfRPgNbDlU5ujWVe5gh5cczas7D6-HnSQsE45tXAVMy9rCAdYQ9nATTZ9VNisGIR76Qj1z8hnj7n8MwPmai0ywT0a8MmdA81C16vogtMenhh2qhbSXV7Zs7YyWvvoL6bmLzl10WNRaWAgy_IvtNVz5K7p7mb2ea6zKbWhLSxPpHUAZtjlLGVjUJ1JkyjTL80GnG3K5YdPpReFX2M1Ms9MM-5rvjlglWWxdGzP-KnO50yH-Zda42dfOIqRTyLmPCiLVZ2dntRn64DY-41AeyklwCb3Y1Mr3WkZfSzocBpOYT3xc3KKCo_lFYTRztvxR5dAyx_JDn14p3SuNHFmdmfHMsKc8K8hwOrfVihwiGaI5LfXsnj62ZAYjRGYQRL0DhGUixxoDB8T5AszmAEwbI48m_LcCmzrs_qx-ctAUX92K4MP0x0iSmy8McfDt0PnKL2JMVQb7__F-uYAHg:1vczoQ:W7u39F0hkrR3yHtsPDxTKFCrbD3ZiDLMzvo9I4t2dt8; OptanonConsent=isGpcEnabled=0^&datestamp=Tue+Jan+06+2026+18^%^3A40^%^3A47+GMT^%^2B1300+(New+Zealand+Daylight+Time)^&version=202409.1.0^&browserGpcFlag=1^&isIABGlobal=false^&hosts=^&genVendors=^&consentId=b61e72d3-0bba-417f-86a2-bf7172919dd8^&interactionCount=1^&isAnonUser=1^&landingPath=NotLandingPage^&groups=1^%^3A1^%^2C2^%^3A1^%^2C3^%^3A1^%^2C4^%^3A1^%^2C5^%^3A1^&AwaitingReconsent=false^&geolocation=NZ^%^3BBOP; lfmjs=1; utag_main=v_id:0199114eb605002080d8c81545e005050001400d00bd0^$_sn:192^$_ss:0^$_st:1767679858082^$vapi_domain:last.fm^$_pn:3^%^3Bexp-session^$ses_id:1767678023159^%^3Bexp-session; AMCV_10D31225525FF5790A490D4D^%^40AdobeOrg=1585540135^%^7CMCIDTS^%^7C20459^%^7CMCMID^%^7C62653102082304541133851090271827602338^%^7CMCAID^%^7CNONE^%^7CMCOPTOUT-1767685258s^%^7CNONE^%^7CvVersion^%^7C4.4.0; s_getNewRepeat=1767678058078-Repeat; s_lv_undefined=1767678058078; csrftoken=9FR3UUxQo9ymVVvmgIqJTdewG7RIk4Hr; OptanonAlertBoxClosed=2026-01-06T05:40:47.065Z; cbsiaa=30504029251034927401623918022991; s_vnum=1770069570723^%^26vn^%^3D7; not_first_visit=1; lfmanon=0; AMCVS_10D31225525FF5790A490D4D^%^40AdobeOrg=1; prevPageType=user_playlists_playlist_overview; s_cc=true; dw-tag=top-tracks; s_sq=^%^5B^%^5BB^%^5D^%^5D; lpfrmo=0; X-UA-Device-Type=desktop; X-UA-Country-Code=NZ; s_invisit=true; s_lv_undefined_s=Less^%^20than^%^201^%^20day^" ^
  -H ^"Sec-Fetch-Dest: empty^" ^
  -H ^"Sec-Fetch-Mode: cors^" ^
  -H ^"Sec-Fetch-Site: same-origin^" ^
  -H ^"DNT: 1^" ^
  -H ^"Pragma: no-cache^" ^
  -H ^"Cache-Control: no-cache^"