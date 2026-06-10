import { createRouter, createWebHistory } from 'vue-router'
import DeepSeek from '../views/DeepSeek.vue'
import Home from '../views/Home.vue'
import Tavern from '../views/Tavern.vue'
import Versions from '../views/Versions.vue'
import Extensions from '../views/Extensions.vue'
import Tools from '../views/Tools.vue'
import Resources from '../views/Resources.vue'
import Console from '../views/Console.vue'
import Settings from '../views/Settings.vue'
import ApiConfig from '../views/ApiConfig.vue'

const routes = [
  {
    path: '/deepseek',
    name: 'DeepSeek',
    component: DeepSeek,
  },
  {
    path: '/',
    name: 'Home',
    component: Home,
  },
  {
    path: '/tavern',
    name: 'Tavern',
    component: Tavern,
  },
  {
    path: '/versions',
    name: 'Versions',
    component: Versions,
  },
  {
    path: '/extensions',
    name: 'Extensions',
    component: Extensions,
  },
  {
    path: '/tools',
    name: 'Tools',
    component: Tools,
  },
  {
    path: '/resources',
    name: 'Resources',
    component: Resources,
  },
  {
    path: '/console',
    name: 'Console',
    component: Console,
  },
  {
    path: '/settings',
    name: 'Settings',
    component: Settings,
  },
  {
    path: '/api-config',
    name: 'ApiConfig',
    component: ApiConfig,
  },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

export default router
