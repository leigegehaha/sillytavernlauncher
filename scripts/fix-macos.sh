#!/bin/bash
# ===================================================
#  Tavern Deepseek Launcher — macOS 修复脚本
#  解决"文件已损坏，无法打开"错误
# ===================================================

set -e

APP_PATH="/Applications/Tavern Deepseek.app"
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo ""
echo -e "${YELLOW}🍺 Tavern Deepseek Launcher — macOS 修复工具${NC}"
echo ""

# 检查 App 是否存在
if [ ! -d "$APP_PATH" ]; then
    echo -e "${RED}❌ 未找到「Tavern Deepseek.app」${NC}"
    echo "   请确认已将 App 拖入 Applications 文件夹。"
    echo ""
    echo "   如果 App 在其他位置，请手动运行："
    echo "   sudo xattr -rd com.apple.quarantine \"/你的路径/Tavern Deepseek.app\""
    exit 1
fi

echo "✅ 已找到：$APP_PATH"
echo ""

# 检查是否有 quarantine 标记
QUARANTINE=$(xattr -l "$APP_PATH" 2>/dev/null | grep "com.apple.quarantine" || true)

if [ -z "$QUARANTINE" ]; then
    echo -e "${GREEN}✅ App 没有 quarantine 隔离标记${NC}"
    echo "   如果仍然打不开，请尝试："
    echo "   系统设置 → 隐私与安全性 → 点击「仍要打开」"
    exit 0
fi

echo -e "${YELLOW}🔍 检测到 quarantine 隔离标记，正在移除...${NC}"
echo ""

# 移除隔离标记（需要 sudo 权限）
echo "🔑 请输入管理员密码："
sudo xattr -rd com.apple.quarantine "$APP_PATH"

echo ""
echo -e "${GREEN}✅ 隔离标记已移除！${NC}"
echo ""
echo "   现在可以正常打开「Tavern Deepseek.app」了 🍺"
echo "   如果弹窗还没消失，右键点击 App → 选择「打开」即可。"
echo ""
