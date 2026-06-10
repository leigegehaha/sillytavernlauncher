<template>
  <div class="h-full flex flex-col gap-4">
    <!-- 页面 Header -->
    <div class="shrink-0 px-4 py-5 rounded-2xl" style="background: linear-gradient(135deg, rgba(34, 26, 20, 0.95), rgba(45, 31, 20, 0.9)); border: 1px solid rgba(180, 140, 100, 0.2);">
      <div class="flex items-center justify-between flex-wrap gap-4">
        <div class="flex items-center gap-4">
          <div class="w-12 h-12 rounded-xl flex items-center justify-center overflow-hidden" style="box-shadow: 0 0 15px rgba(230, 180, 34, 0.2);">
            <img src="../assets/logo.png" alt="DeepSeek Tavern" class="w-full h-full object-cover" />
          </div>
          <div>
            <h1 class="text-xl font-bold" style="font-family: Georgia, 'Times New Roman', serif; color: #e6b422; text-shadow: 0 0 10px rgba(230, 180, 34, 0.2);">
              {{ t('deepseek.title') }}
            </h1>
            <p class="text-sm text-parchment-400 mt-0.5">酒馆专属大模型服务 · <span style="color: #e6b422;">点击按钮打开网站获取 API Key</span></p>
          </div>
        </div>
        <a
          href="https://deepseektavern.com/"
          target="_blank"
          class="deepseek-site-btn px-5 py-3 rounded-xl text-sm font-bold shadow-md active:scale-95 flex items-center gap-2"
        >
          <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path>
            <polyline points="15 3 21 3 21 9"></polyline>
            <line x1="10" y1="14" x2="21" y2="3"></line>
          </svg>
          酒馆专属大模型网站
        </a>
      </div>
    </div>

    <!-- Tavern Account -->
    <TavernAccount />

  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import TavernAccount from '../components/TavernAccount.vue'

const { t } = useI18n()
</script>

<style scoped>
.deepseek-site-btn {
  background: linear-gradient(135deg, #8b6914, #d4a017);
  color: #fdf4dc;
  box-shadow: 0 0 20px rgba(230, 180, 34, 0.2);
  transition: all 0.3s ease;
}
.deepseek-site-btn:hover {
  box-shadow: 0 0 35px rgba(230, 180, 34, 0.35);
  transform: translateY(-1px);
}
</style>
