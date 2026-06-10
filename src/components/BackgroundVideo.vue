<template>
  <div class="fixed inset-0 overflow-hidden pointer-events-none" style="z-index: 0; background: #120c08;">
    <!-- Video for all pages -->
    <video
      ref="videoRef"
      autoplay
      muted
      loop
      playsinline
      preload="auto"
      class="absolute inset-0 w-full h-full object-cover transition-opacity duration-1000"
      :style="{ opacity: videoReady ? 0.50 : 0 }"
    >
      <source :src="currentVideo" type="video/mp4" />
    </video>
    <div class="absolute inset-0" style="background: linear-gradient(180deg, rgba(18, 12, 8, 0.55) 0%, rgba(18, 12, 8, 0.45) 50%, rgba(18, 12, 8, 0.55) 100%);"></div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch, onMounted } from 'vue'
import { useRoute } from 'vue-router'

const route = useRoute()
const videoRef = ref<HTMLVideoElement | null>(null)
const videoReady = ref(false)

const videos = [
  '/videos/bg-hero.mp4',
  '/videos/bg-testimonial.mp4',
  '/videos/bg-interior.mp4',
  '/videos/bg-library.mp4',
]

const videoMap: Record<string, string> = {
  Home: videos[1],
  Tavern: videos[2],
  DeepSeek: videos[3],
  Versions: videos[3],
  Extensions: videos[0],
  Resources: videos[1],
  Console: videos[2],
  Settings: videos[3],
  Tools: videos[0],
}

const currentVideo = computed(() => {
  const name = route.name as string
  return videoMap[name] || videos[0]
})

const onCanPlay = () => {
  videoReady.value = true
}

const onWaiting = () => {
  videoReady.value = false
}

watch(currentVideo, () => {
  videoReady.value = false
  if (videoRef.value) {
    videoRef.value.load()
  }
})

onMounted(() => {
  if (videoRef.value) {
    videoRef.value.addEventListener('canplay', onCanPlay)
    videoRef.value.addEventListener('waiting', onWaiting)
  }
})
</script>
