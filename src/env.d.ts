// 声明前端构建环境的类型边界，保证 TS 能识别 Vite 与 SFC 类型。
/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly GOLUTRA_Front_DEBUG?: string;
}

declare module '*.vue' {
  import type { DefineComponent } from 'vue';
  // 统一的 SFC 默认导出类型，避免在业务代码中显式声明组件类型。
  const component: DefineComponent<Record<string, unknown>, Record<string, unknown>, unknown>;
  export default component;
}
