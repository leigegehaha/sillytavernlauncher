import pkg from '../../package.json'
import {
  PhOpenAiLogo,
  PhGoogleLogo,
  PhStorefront,
  PhBookOpen,
  PhDeviceMobile,
  PhKey,
  PhMicrophone,
  PhRobot,
  PhGithubLogo,
  PhBooks,
} from '@phosphor-icons/vue'
import logo from '../assets/logo.png'

export default {
  appName: 'Tavern Deepseek',
  appNameEn: 'Tavern Deepseek',
  appVersion: pkg.version,
  appDescription: pkg.description,
  appDescriptionEn: pkg.descriptionEn,
  appHomepage: pkg.homepage,
  appIcon: logo,
  git: {
    github: 'https://github.com/al01cn/sillyTavern-launcher',
    gitee: 'https://gitee.com/al01/sillytavern-launcher',
  },
  tools: {
    酒馆资源: [
      {
        name: '酒馆专属模型',
        url: 'https://deepseektavern.com/',
        desc: '不光有酒馆专属模型，还有 Grok、Gemini、Claude，还有语音生成模型等，价格白菜价。',
        icon: 'https://index.deepseektavern.com/logo.png',
      },
      {
        name: 'SillyTavern 模型配置教程',
        url: 'https://docs.deepseektavern.com/sillytavern-config.html',
        desc: '手把手教你在酒馆中配置 DeepSeek Tavern 等大模型。',
        defaultIcon: PhBookOpen,
      },
      {
        name: '手机版酒馆 Tavo 配置教程',
        url: 'https://docs.deepseektavern.com/tavo-mobile-guide.html',
        desc: '在手机上也能玩酒馆，Tavo 移动端配置全攻略。',
        defaultIcon: PhDeviceMobile,
      },
      {
        name: 'API 密钥获取教程',
        url: 'https://docs.deepseektavern.com/api-key-guide.html',
        desc: '注册获取 API 密钥，接入酒馆专属大模型。',
        defaultIcon: PhKey,
      },
      {
        name: 'TTS 语音合成教程（让角色卡说话）',
        url: 'https://docs.deepseektavern.com/api-key-guide.html',
        desc: '给角色卡装上嘴巴，文字转语音让对话更生动。',
        defaultIcon: PhMicrophone,
      },
      {
        name: 'AI Agent 完全入门指南',
        url: 'https://agent.deepseektavern.com/',
        desc: '从零开始了解 AI Agent，自动化你的工作流。',
        defaultIcon: PhRobot,
      },
      {
        name: '酒馆角色卡蒸馏器（微信上玩酒馆）',
        url: 'https://github.com/leigegehaha/tavern-card-distiller',
        desc: '在微信上也能管理酒馆角色卡，移动端轻量方案。',
        defaultIcon: PhGithubLogo,
      },
      {
        name: '更多酒馆教程',
        url: 'https://docs.deepseektavern.com/sillytavern-install.html',
        desc: 'SillyTavern 安装、配置、使用的完整教程合集。',
        defaultIcon: PhBooks,
      },
      {
        name: '酒馆 Wiki',
        url: 'https://sillytavern.wiki/',
        desc: 'SillyTavern 中文社区 Wiki，最全的酒馆文档。',
        icon: 'https://sillytavern.wiki/favicon.ico',
        defaultIcon: PhStorefront,
      },
    ],
  },
  ca: {
    categories: [
      {
        name: '框架 / Frameworks',
        items: [
          { name: 'Tauri', version: '2', url: 'https://tauri.app/', key: 'tauri' },
          { name: 'Vue', version: '3.5', url: 'https://vuejs.org/', key: 'vue' },
          { name: 'Rust', version: '1.75+', url: 'https://www.rust-lang.org/', key: 'rust' },
        ],
      },
      {
        name: '前端依赖 / Frontend',
        items: [
          { name: 'Tailwind CSS', version: '4.2', url: 'https://tailwindcss.com/', key: 'tailwind' },
          { name: 'vue-i18n', version: '11', url: 'https://vue-i18n.intlify.dev/', key: 'vueI18n' },
          { name: 'vue-router', version: '5', url: 'https://router.vuejs.org/', key: 'vueRouter' },
          { name: 'Phosphor Icons', version: '2.2', url: 'https://phosphoricons.com/', key: 'phosphorIcons' },
          { name: 'Lucide Vue', version: '0.577', url: 'https://lucide.dev/', key: 'lucide' },
          { name: 'QRCode', version: '1.5', url: 'https://github.com/soldair/node-qrcode', key: 'qrcode' },
          { name: 'Vue Sonner', version: '2', url: 'https://github.com/AntonyAnu/sonner-vue', key: 'vueSonner' },
          { name: 'DaisyUI', version: '5', url: 'https://daisyui.com/', key: 'daisyui' },
        ],
      },
      {
        name: '后端依赖 / Backend',
        items: [
          { name: 'Tokio', version: '1', url: 'https://tokio.rs/', key: 'tokio' },
          { name: 'Reqwest', version: '0.12', url: 'https://docs.rs/reqwest/0.12/reqwest/', key: 'reqwest' },
          { name: 'Serde', version: '1', url: 'https://serde.rs/', key: 'serde' },
          { name: 'Zip', version: '0.6', url: 'https://github.com/zip-rs/zip', key: 'zip' },
          { name: 'Walkdir', version: '2.5', url: 'https://github.com/BurntSushi/walkdir', key: 'walkdir' },
          { name: 'Jwalk', version: '0.8', url: 'https://github.com/Byron/jwalk', key: 'jwalk' },
          { name: 'Sevenz', version: '0.6', url: 'https://github.com/erthink/7z', key: 'sevenz' },
          {
            name: 'Headless Chrome',
            version: '1',
            url: 'https://github.com/ChromeDevTools/headless_chrome',
            key: 'headlessChrome',
          },
          { name: 'Winreg', version: '0.52', url: 'https://github.com/gentoo90/winreg', key: 'winreg' },
        ],
      },
      {
        name: '开发工具 / DevTools',
        items: [
          { name: 'Vite', version: '6', url: 'https://vite.dev/', key: 'vite' },
          { name: 'TypeScript', version: '6', url: 'https://www.typescriptlang.org/', key: 'typescript' },
          { name: 'ESLint', version: '10', url: 'https://eslint.org/', key: 'eslint' },
          { name: 'Prettier', version: '3.8', url: 'https://prettier.io/', key: 'prettier' },
        ],
      },
      {
        name: '特别感谢 / Special Thanks',
        items: [
          { name: 'SillyTavern', version: '1.1x.x', url: 'https://sillytavern.app/', key: 'sillytavern' },
          {
            name: 'SillyTavern 社区',
            version: '',
            url: 'https://github.com/SillyTavern/SillyTavern',
            key: 'sillytavernCommunity',
          },
          {
            name: 'Github Proxy',
            version: '',
            url: 'https://github.akams.cn/',
            key: 'githubProxy',
          },
          {
            name: 'Github Proxy - ghfast',
            version: '',
            url: 'https://ghfast.top/',
            key: 'ghfast',
          },
        ],
      },
    ],
  },
}
