<script setup lang="ts">
import { onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import {
  echo as echoFromOhosDemo,
  getPlatformInfo as getOhosDemoPlatformInfo,
  type OhosDemoPlatformInfo,
} from "../src-tauri/tauri-plugin-ohos-demo/guest-js";

interface RegressionCheck {
  name: string;
  ok: boolean;
  detail: string;
}

interface RegressionReport {
  passed: number;
  failed: number;
  checks: RegressionCheck[];
}

const greetMsg = ref("");
const name = ref("");
const pluginMessage = ref("hello from Vue");
const pluginEcho = ref("");
const pluginInfo = ref<OhosDemoPlatformInfo | null>(null);
const arktsMessage = ref("hello from frontend");
const arktsFunctionResult = ref("");
const arktsTsfnResult = ref("");
const regressionReport = ref<RegressionReport | null>(null);
const frontendChecks = ref<RegressionCheck[]>([]);
const dialogProbeResult = ref("");

async function greet() {
  greetMsg.value = await invoke("greet", { name: name.value });
}

async function loadPluginInfo() {
  pluginInfo.value = await getOhosDemoPlatformInfo();
}

async function runPluginEcho() {
  pluginEcho.value = await echoFromOhosDemo(pluginMessage.value);
}

async function callArktsFunction() {
  arktsFunctionResult.value = "calling...";
  try {
    arktsFunctionResult.value = await invoke("call_arkts_function_from_frontend", {
      message: arktsMessage.value,
    });
  } catch (error) {
    arktsFunctionResult.value = `error: ${String(error)}`;
  }
}

async function callArktsTsfn() {
  arktsTsfnResult.value = "calling...";
  try {
    arktsTsfnResult.value = await invoke("call_arkts_tsfn_from_frontend", {
      message: arktsMessage.value,
    });
  } catch (error) {
    arktsTsfnResult.value = `error: ${String(error)}`;
  }
}

async function runDialogProbe() {
  dialogProbeResult.value = "opening...";
  try {
    dialogProbeResult.value = await invoke("trigger_dialog_message_probe");
  } catch (error) {
    dialogProbeResult.value = `error: ${String(error)}`;
  }
}

async function runRegressionSuite() {
  const fe: RegressionCheck[] = [];
  try {
    const g = await invoke<string>("greet", { name: "OHOS" });
    fe.push({ name: "ipc-greet", ok: g.includes("OHOS"), detail: g });
  } catch (e) {
    fe.push({ name: "ipc-greet", ok: false, detail: String(e) });
  }

  try {
    const info = await getOhosDemoPlatformInfo();
    pluginInfo.value = info;
    const echoed = await echoFromOhosDemo("auto-probe");
    pluginEcho.value = echoed;
    fe.push({
      name: "js-ohos-demo-plugin",
      ok: echoed.includes("auto-probe"),
      detail: `${info.platform}/${info.runtime} echo=${echoed}`,
    });
  } catch (e) {
    fe.push({ name: "js-ohos-demo-plugin", ok: false, detail: String(e) });
  }

  try {
    localStorage.setItem("tauri_ohos_probe", "ok");
    const val = localStorage.getItem("tauri_ohos_probe");
    fe.push({ name: "webview-dom-storage", ok: val === "ok", detail: `val=${val}` });
  } catch (e) {
    fe.push({ name: "webview-dom-storage", ok: false, detail: String(e) });
  }

  try {
    const report = await invoke<RegressionReport>("run_ohos_regression_checks");
    regressionReport.value = report;
  } catch (e) {
    fe.push({ name: "rust-regression-suite", ok: false, detail: String(e) });
  }

  await new Promise((r) => setTimeout(r, 300));
  const evalVal = (window as unknown as Record<string, unknown>).__wryEvalProbe;
  fe.push({
    name: "wry-eval-js-side-effect",
    ok: evalVal === "ok",
    detail: `__wryEvalProbe=${String(evalVal)}`,
  });

  frontendChecks.value = fe;
}

onMounted(() => {
  void runRegressionSuite();
});
</script>

<template>
  <main class="container">
    <h1>Welcome to Tauri + Vue (OpenHarmony)</h1>

    <div class="row">
      <a href="https://vite.dev" target="_blank">
        <img src="/vite.svg" class="logo vite" alt="Vite logo" />
      </a>
      <a href="https://tauri.app" target="_blank">
        <img src="/tauri.svg" class="logo tauri" alt="Tauri logo" />
      </a>
      <a href="https://vuejs.org/" target="_blank">
        <img src="./assets/vue.svg" class="logo vue" alt="Vue logo" />
      </a>
    </div>

    <section class="regression-demo">
      <h2>OHOS Regression Suite</h2>
      <div class="row">
        <button type="button" @click="runRegressionSuite">Re-run Checks</button>
        <button type="button" @click="runDialogProbe">Test Dialog Prompt</button>
      </div>
      <p v-if="dialogProbeResult">{{ dialogProbeResult }}</p>
      <p v-if="regressionReport">
        Rust checks: {{ regressionReport.passed }} passed / {{ regressionReport.failed }} failed
      </p>
      <ul class="check-list">
        <li v-for="c in frontendChecks" :key="c.name" :class="c.ok ? 'pass' : 'fail'">
          [{{ c.ok ? "PASS" : "FAIL" }}] {{ c.name }}: {{ c.detail }}
        </li>
        <li
          v-for="c in regressionReport?.checks ?? []"
          :key="c.name"
          :class="c.ok ? 'pass' : 'fail'"
        >
          [{{ c.ok ? "PASS" : "FAIL" }}] {{ c.name }}: {{ c.detail }}
        </li>
      </ul>
    </section>

    <form class="row" @submit.prevent="greet">
      <input id="greet-input" v-model="name" placeholder="Enter a name..." />
      <button type="submit">Greet</button>
    </form>
    <p>{{ greetMsg }}</p>

    <section class="plugin-demo">
      <h2>OHOS plugin example</h2>
      <div class="row">
        <button type="button" @click="loadPluginInfo">Load plugin info</button>
      </div>
      <p v-if="pluginInfo">
        {{ pluginInfo.platform }} / {{ pluginInfo.runtime }} /
        NativeAbility: {{ pluginInfo.usesNativeAbility ? "yes" : "no" }}
      </p>

      <form class="row" @submit.prevent="runPluginEcho">
        <input v-model="pluginMessage" placeholder="Plugin message..." />
        <button type="submit">Plugin echo</button>
      </form>
      <p>{{ pluginEcho }}</p>
    </section>

    <section class="arkts-demo">
      <h2>ArkTS callback example</h2>
      <form class="row" @submit.prevent="callArktsFunction">
        <input v-model="arktsMessage" placeholder="ArkTS message..." />
        <button type="submit">ArkTS Function</button>
      </form>
      <p>{{ arktsFunctionResult }}</p>

      <div class="row">
        <button type="button" @click="callArktsTsfn">ArkTS TSFN</button>
      </div>
      <p>{{ arktsTsfnResult }}</p>
    </section>
  </main>
</template>

<style scoped>
.logo.vite:hover {
  filter: drop-shadow(0 0 2em #747bff);
}

.logo.vue:hover {
  filter: drop-shadow(0 0 2em #249b73);
}

.check-list {
  list-style: none;
  padding: 0 16px;
  text-align: left;
  font-size: 13px;
  line-height: 1.5;
  max-width: 760px;
  margin: 12px auto;
}

.check-list li.pass {
  color: #1a7f37;
}

.check-list li.fail {
  color: #cf222e;
  font-weight: 600;
}
</style>
<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color: #0f0f0f;
  background-color: #f6f6f6;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

.container {
  margin: 0;
  padding: 4vh 16px 8vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;
}

.logo {
  height: 4.5em;
  padding: 1em;
  will-change: filter;
  transition: 0.75s;
}

.logo.tauri:hover {
  filter: drop-shadow(0 0 2em #24c8db);
}

.row {
  display: flex;
  justify-content: center;
  gap: 5px;
}

a {
  font-weight: 500;
  color: #646cff;
  text-decoration: inherit;
}

a:hover {
  color: #535bf2;
}

h1 {
  text-align: center;
}

input,
button {
  border-radius: 8px;
  border: 1px solid transparent;
  padding: 0.6em 1.2em;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  color: #0f0f0f;
  background-color: #ffffff;
  transition: border-color 0.25s;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
}

button {
  cursor: pointer;
}

button:hover {
  border-color: #396cd8;
}
button:active {
  border-color: #396cd8;
  background-color: #e8e8e8;
}

input,
button {
  outline: none;
}

#greet-input {
  margin-right: 5px;
}

.regression-demo,
.plugin-demo,
.arkts-demo {
  margin-top: 1.5rem;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }

  a:hover {
    color: #24c8db;
  }

  input,
  button {
    color: #ffffff;
    background-color: #0f0f0f98;
  }
  button:active {
    background-color: #0f0f0f69;
  }
}
</style>
