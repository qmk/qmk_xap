import { LogicalSize, PhysicalPosition, currentMonitor, getCurrent } from '@tauri-apps/api/window'

interface FitWindowOptions {
    contentWidth: number
    contentHeight: number
    minWidth?: number
    minHeight?: number
    monitorMargin?: number
}

function hasTauriWindowApi(): boolean {
    return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

function clamp(value: number, min: number, max: number): number {
    return Math.min(Math.max(value, min), max)
}

export async function fitCurrentWindowToContent({
    contentWidth,
    contentHeight,
    minWidth = 800,
    minHeight = 600,
    monitorMargin = 64,
}: FitWindowOptions): Promise<void> {
    if (!hasTauriWindowApi() || contentWidth <= 0 || contentHeight <= 0) return

    try {
        const appWindow = getCurrent()
        const scaleFactor = await appWindow.scaleFactor()
        const [innerSize, outerSize, monitor] = await Promise.all([
            appWindow.innerSize(),
            appWindow.outerSize(),
            currentMonitor(),
        ])
        const innerLogical = innerSize.toLogical(scaleFactor)
        const outerLogical = outerSize.toLogical(scaleFactor)
        const chromeWidth = Math.max(outerLogical.width - innerLogical.width, 0)
        const chromeHeight = Math.max(outerLogical.height - innerLogical.height, 0)
        const monitorLogical = monitor?.size.toLogical(monitor.scaleFactor)
        // Hard cap: the window (outer = inner + chrome) must never exceed the
        // monitor. On a monitor smaller than minWidth/minHeight the cap wins, so
        // the floor is clamped below the cap rather than pushing past the screen.
        const maxInnerWidth = monitorLogical
            ? Math.max(0, monitorLogical.width - monitorMargin - chromeWidth)
            : Number.POSITIVE_INFINITY
        const maxInnerHeight = monitorLogical
            ? Math.max(0, monitorLogical.height - monitorMargin - chromeHeight)
            : Number.POSITIVE_INFINITY
        const targetInnerWidth = Math.ceil(
            clamp(contentWidth, Math.min(minWidth, maxInnerWidth), maxInnerWidth),
        )
        const targetInnerHeight = Math.ceil(
            clamp(contentHeight, Math.min(minHeight, maxInnerHeight), maxInnerHeight),
        )
        const targetOuterWidth = targetInnerWidth + chromeWidth
        const targetOuterHeight = targetInnerHeight + chromeHeight

        if (
            Math.abs(outerLogical.width - targetOuterWidth) < 2 &&
            Math.abs(outerLogical.height - targetOuterHeight) < 2
        ) {
            return
        }

        const outerPosition = await appWindow.outerPosition()
        const previousCenterX = outerPosition.x + outerSize.width / 2
        const previousCenterY = outerPosition.y + outerSize.height / 2

        await appWindow.setSize(new LogicalSize(targetOuterWidth, targetOuterHeight))

        // Keep auto-fit from visually pinning growth and shrinkage to the top-left corner.
        const resizedOuterSize = await appWindow.outerSize()
        await appWindow.setPosition(
            new PhysicalPosition(
                Math.round(previousCenterX - resizedOuterSize.width / 2),
                Math.round(previousCenterY - resizedOuterSize.height / 2),
            ),
        )
    } catch (error) {
        console.debug('Skipping automatic window fit', error)
    }
}
