:: arm7 doesn't work with aubio
dx bundle --platform android --release --package trackfish --target x86_64-linux-android
dx bundle --platform android --release --package trackfish --target aarch64-linux-android

:: both .aab files are the same
rm ./apk/trackfish.apks
java -jar "C:\bundletool\bundletool.jar" build-apks --bundle=./target/dx/trackfish/release/android/app/app/build/outputs/bundle/release/Trackfish-x86_64-linux-android.aab --output=./apk/trackfish.apks --mode universal

:: sign apk and verify
apksigner sign --ks my-release-key.jks --ks-key-alias my-key-alias apk/trackfish/trackfish.apk
apksigner "apk/trackfish/trackfish.apk
