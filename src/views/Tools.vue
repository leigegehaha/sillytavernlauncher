<template>
  <div class="space-y-8 pb-10">
    <div v-for="(toolList, category) in tools" :key="category" class="space-y-2">
      <h2 class="text-lg font-bold text-slate-700 border-b border-slate-100 pb-2">
        {{ category }}
      </h2>
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <button
          v-for="(tool, index) in toolList"
          :key="index"
          class="group p-5 rounded-2xl border transition-all duration-300 flex items-start gap-4 text-left tavern-card"
          style="border-color: rgba(180, 140, 100, 0.15);"
          @click="openLink(tool.url)"
          @mouseenter="(e: any) => { e.target.style.borderColor = 'rgba(230, 180, 34, 0.4)'; e.target.style.boxShadow = '0 0 25px rgba(230, 180, 34, 0.1)'; }"
          @mouseleave="(e: any) => { e.target.style.borderColor = 'rgba(180, 140, 100, 0.15)'; e.target.style.boxShadow = 'none'; }"
        >
          <div
            class="w-12 h-12 rounded-xl flex items-center justify-center shrink-0 transition-transform duration-300 overflow-hidden"
            style="background: rgba(180, 140, 100, 0.15);"
            :class="{ 'group-hover:scale-110': true }"
          >
            <img
              v-if="tool.icon && !imageErrors[`${category}-${index}`]"
              :src="tool.icon"
              :alt="tool.name"
              class="w-full h-full object-cover"
              @error="handleImageError(category, index)"
            />
            <component :is="tool.defaultIcon" v-else-if="tool.defaultIcon" class="w-6 h-6" style="color: #e6b422;" />
            <Zap v-else class="w-6 h-6" style="color: #e6b422;" />
          </div>
          <div class="flex-1 min-w-0 pt-1">
            <h3 class="text-base font-bold mb-1 transition-colors" style="color: #e0c8a0; font-family: Georgia, 'Times New Roman', serif;">
              {{ tool.name }}
            </h3>
            <p class="text-xs" style="color: #9e7a5c; line-height: 1.5;">
              {{ (tool as any).desc || tool.url }}
            </p>
          </div>
          <div class="pt-2 transition-colors" style="color: #9e7a5c;">
            <ExternalLink class="w-4 h-4" />
          </div>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Wrench, ExternalLink, Zap } from 'lucide-vue-next'
import { openUrl as open } from '@tauri-apps/plugin-opener'
// import { useI18n } from 'vue-i18n'; // 暂未使用
import config from '../lib/config'
import { getCachedImageUrl } from '../lib/imageCache'

// const { t } = useI18n(); // 暂未使用

interface ToolItem {
  name: string
  url: string
  icon?: string
  defaultIcon?: any
}

const tools = ref<Record<string, ToolItem[]>>((config.tools as Record<string, ToolItem[]>) || {})
const imageErrors = ref<Record<string, boolean>>({})

const handleImageError = (category: string | number, index: number) => {
  imageErrors.value[`${category}-${index}`] = true
}

const openLink = async (url: string) => {
  try {
    await open(url)
  } catch (error) {
    console.error('Failed to open link:', error)
    // Fallback to window.open if plugin-shell fails
    window.open(url, '_blank')
  }
}

onMounted(async () => {
  // 遍历所有工具并缓存图标
  const toolsData = { ...tools.value }

  for (const category in toolsData) {
    for (let i = 0; i < toolsData[category].length; i++) {
      const tool = toolsData[category][i]
      if (tool.icon) {
        // 异步获取缓存的图片 URL
        getCachedImageUrl(tool.icon)
          .then(cachedUrl => {
            tool.icon = cachedUrl
          })
          .catch(err => {
            console.warn('Failed to cache icon for', tool.name, err)
          })
      }
    }
  }
})
</script>
