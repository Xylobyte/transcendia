/* Copyright © 2025 Xylobyte
 * SPDX-License-Identifier: AGPL-3.0-or-later */

import {createApp} from "vue";
import App from "./Config.vue";
import "../index.css";
import Vue3ColorPicker from "vue3-colorpicker";
import "vue3-colorpicker/style.css";

createApp(App).use(Vue3ColorPicker).mount("#app");
