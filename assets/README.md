# 用于存放 macOS app 图标等资源

# 图标文件 (需要提供)
# icon.icns - macOS 应用图标

# 生成图标的方法：
# 1. 准备一个 1024x1024 的 PNG 图片
# 2. 使用在线工具或以下命令生成 .icns：
#    mkdir icon.iconset
#    sips -z 16 16     icon.png --out icon.iconset/icon_16x16.png
#    sips -z 32 32     icon.png --out icon.iconset/icon_16x16@2x.png
#    sips -z 32 32     icon.png --out icon.iconset/icon_32x32.png
#    sips -z 64 64     icon.png --out icon.iconset/icon_32x32@2x.png
#    sips -z 128 128   icon.png --out icon.iconset/icon_128x128.png
#    sips -z 256 256   icon.png --out icon.iconset/icon_128x128@2x.png
#    sips -z 256 256   icon.png --out icon.iconset/icon_256x256.png
#    sips -z 512 512   icon.png --out icon.iconset/icon_256x256@2x.png
#    sips -z 512 512   icon.png --out icon.iconset/icon_512x512.png
#    sips -z 1024 1024 icon.png --out icon.iconset/icon_512x512@2x.png
#    iconutil -c icns icon.iconset -o icon.icns
