// Composables 导出
export { useTheme, THEMES, DEFAULT_THEME } from './useTheme'
export { useLayout, LAYOUTS, DEFAULT_LAYOUT, DEFAULT_SPLITTER_POSITION } from './useLayout'
export type { LayoutState } from './useLayout'
export { useAudio } from './useAudio'
export { useDisplayMode, selectDisplayContent } from './useDisplayMode'
export type { DisplayModeParams, UseDisplayModeReturn } from './useDisplayMode'
export { useImageHandler } from './useImageHandler'
export { 
  useFileHandler, 
  isImageFile, 
  getFileName, 
  getFileExtension, 
  formatFileReference 
} from './useFileHandler'
export { 
  useTextOptimization,
  type OptimizationType,
  type TextOptimizationResult,
  type OptimizationHistoryItem
} from './useTextOptimization'
export { 
  useApiKeys, 
  PROVIDERS,
  type ApiProvider,
  type ProviderInfo
} from './useApiKeys'
export { 
  useScreenshot,
  type ScreenshotResult,
  type SelectionRect
} from './useScreenshot'
export {
  usePlatform,
  detectPlatform,
  getPathSeparator,
  normalizePath,
  joinPath,
  getPrimaryModifier,
  isPasteShortcut,
  isCopyShortcut,
  isUndoShortcut,
  isRedoShortcut,
  isSelectAllShortcut,
  formatShortcut,
  getPasteShortcutText,
  getUndoShortcutText,
  PATH_SEPARATORS,
  PRIMARY_MODIFIER,
  type Platform,
  type ModifierKey,
  type ShortcutConfig
} from './usePlatform'
export {
  useSplitter,
  type SplitterOptions,
  type SplitterReturn
} from './useSplitter'
export {
  useDragDrop,
  type TauriDragDropPayload,
  type DragDropCallbacks,
  type DragDropReturn
} from './useDragDrop'
