<script setup lang="ts">
// Vue harness(Plan 34 S5 / research R8):同一份 wasm pkg + 同一套桥(bootCanvas),
// 框架只提供壳(模板/生命周期)。**零改 wasm API** —— 改了即框架耦合,判负。
import { onMounted, ref } from "vue";
import { bootCanvas } from "../../../src/boot";

const status = ref("booting…");
const openedUrl = ref("");

onMounted(async () => {
  const { chat } = await bootCanvas({ findBar: false });
  // 与主 harness 同款调试句柄(e2e smoke 复用同一套探针)。
  (window as unknown as { __chat: unknown }).__chat = chat;
  chat.set_open_url_handler((url: string) => {
    openedUrl.value = url; // Vue 响应式收回调 —— 宿主决定怎么处理(这里仅展示)
  });
  const tick = () => {
    status.value = `${chat.session_status()} · ${chat.follow_state()}`;
    requestAnimationFrame(tick);
  };
  tick();
});
</script>

<template>
  <header class="bar">
    <strong>infinite-chat · Vue 壳</strong>
    <span data-test="status">{{ status }}</span>
    <span v-if="openedUrl" data-test="opened-url">{{ openedUrl }}</span>
  </header>
  <canvas id="chat"></canvas>
</template>

<style scoped>
.bar {
  display: flex;
  gap: 12px;
  padding: 6px 12px;
  color: #a0a0a0;
  font: 12px system-ui;
  background: #1d1d20;
}
</style>
