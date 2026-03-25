import { getCurrentWindow, currentMonitor, PhysicalPosition, PhysicalSize } from "@tauri-apps/api/window";

type Bounds = { x: number; y: number; width: number; height: number };

function easeOutQuint(t: number): number {
  return 1 - Math.pow(1 - t, 5);
}

async function animateWindow(from: Bounds, to: Bounds, duration = 700): Promise<void> {
  const win = getCurrentWindow();
  const startTime = performance.now();

  return new Promise((resolve) => {
    function step(currentTime: number) {
      const progress = Math.min((currentTime - startTime) / duration, 1);
      const eased = easeOutQuint(progress);

      win.setPosition(new PhysicalPosition(
        Math.round(from.x + (to.x - from.x) * eased),
        Math.round(from.y + (to.y - from.y) * eased),
      ));
      win.setSize(new PhysicalSize(
        Math.round(from.width + (to.width - from.width) * eased),
        Math.round(from.height + (to.height - from.height) * eased),
      ));

      if (progress < 1) {
        requestAnimationFrame(step);
      } else {
        resolve();
      }
    }
    requestAnimationFrame(step);
  });
}

export function createMaximizeToggle() {
  let savedBounds: Bounds | null = null;
  let isMaximized = false;
  let isAnimating = false;

  return async function toggleMaximize() {
    if (isAnimating) return;

    const win = getCurrentWindow();
    const monitor = await currentMonitor();
    if (!monitor) return;

    isAnimating = true;

    const pos = await win.outerPosition();
    const size = await win.outerSize();
    const current = { x: pos.x, y: pos.y, width: size.width, height: size.height };

    if (isMaximized && savedBounds) {
      await animateWindow(current, savedBounds);
      isMaximized = false;
      savedBounds = null;
    } else {
      savedBounds = current;
      const { position, size: monSize } = monitor;
      await animateWindow(current, { x: position.x, y: position.y, width: monSize.width, height: monSize.height });
      isMaximized = true;
    }

    isAnimating = false;
  };
}
