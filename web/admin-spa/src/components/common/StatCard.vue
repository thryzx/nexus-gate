<template>
  <div class="p-1 rounded-xl shadow border" :class="cardClass">
    <div class="p-5">
      <p class="text-sm text-gray-500 dark:text-gray-400">{{ title }}</p>
      <p class="text-2xl font-bold mt-1" :class="textClass">{{ value }}</p>
      <p v-if="sub" class="text-xs text-gray-400 dark:text-gray-500 mt-1">{{ sub }}</p>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue'

const props = defineProps({
  title: String,
  value: [String, Number],
  sub: String,
  color: { type: String, default: 'blue' }
})

const colorMap = {
  blue: { card: 'bg-white dark:bg-gray-800 border-blue-100 dark:border-blue-900', text: 'text-blue-600 dark:text-blue-400' },
  green: { card: 'bg-white dark:bg-gray-800 border-green-100 dark:border-green-900', text: 'text-green-600 dark:text-green-400' },
  purple: { card: 'bg-white dark:bg-gray-800 border-purple-100 dark:border-purple-900', text: 'text-purple-600 dark:text-purple-400' },
  orange: { card: 'bg-white dark:bg-gray-800 border-orange-100 dark:border-orange-900', text: 'text-orange-600 dark:text-orange-400' }
}

const cardClass = computed(() => colorMap[props.color]?.card || colorMap.blue.card)
const textClass = computed(() => colorMap[props.color]?.text || colorMap.blue.text)
</script>
