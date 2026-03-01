// 头像渲染工具：将头像绘制为 PNG 字节，供托盘图标使用。
import { ensureAvatar, getAvatarVars, isCssAvatar, resolveAvatarUrl } from '@/shared/utils/avatar';

const parsePercent = (value: string, fallback = 0) => {
  const trimmed = value.trim();
  if (!trimmed.endsWith('%')) {
    const numeric = Number.parseFloat(trimmed);
    return Number.isFinite(numeric) ? numeric / 100 : fallback;
  }
  const numeric = Number.parseFloat(trimmed.slice(0, -1));
  return Number.isFinite(numeric) ? numeric / 100 : fallback;
};

const normalizeColorStop = (value: string) => {
  const trimmed = value.trim();
  const percentMatch = trimmed.match(/^(.*)\s+[-\d.]+%$/);
  if (percentMatch) {
    return percentMatch[1].trim();
  }
  const numberMatch = trimmed.match(/^(.*)\s+[-\d.]+$/);
  if (numberMatch) {
    return numberMatch[1].trim();
  }
  return trimmed;
};

const splitGradientArgs = (value: string) => {
  const start = value.indexOf('(');
  const end = value.lastIndexOf(')');
  if (start === -1 || end === -1 || end <= start) return null;
  const body = value.slice(start + 1, end);
  const args: string[] = [];
  let buffer = '';
  let depth = 0;
  for (const char of body) {
    if (char === '(') {
      depth += 1;
      buffer += char;
      continue;
    }
    if (char === ')') {
      depth = Math.max(0, depth - 1);
      buffer += char;
      continue;
    }
    if (char === ',' && depth === 0) {
      args.push(buffer.trim());
      buffer = '';
      continue;
    }
    buffer += char;
  }
  if (buffer.trim()) {
    args.push(buffer.trim());
  }
  return args;
};

const parseLinearGradient = (value: string) => {
  if (!value.toLowerCase().startsWith('linear-gradient(')) return null;
  const args = splitGradientArgs(value);
  if (!args || args.length < 3) return null;
  const angleText = args[0].trim();
  if (!angleText.endsWith('deg')) return null;
  const angle = Number.parseFloat(angleText);
  if (!Number.isFinite(angle)) return null;
  const start = normalizeColorStop(args[1]);
  const end = normalizeColorStop(args[2]);
  return { angle, start, end };
};

const parseRadialGradient = (value: string) => {
  if (!value.toLowerCase().startsWith('radial-gradient(')) return null;
  const args = splitGradientArgs(value);
  if (!args || args.length < 3) return null;
  const anchor = args[0];
  const match = anchor.match(/circle at\s+([-\d.]+)%\s+([-\d.]+)%/i);
  if (!match) return null;
  const cx = Number.parseFloat(match[1]) / 100;
  const cy = Number.parseFloat(match[2]) / 100;
  if (!Number.isFinite(cx) || !Number.isFinite(cy)) return null;
  const start = normalizeColorStop(args[1]);
  const end = normalizeColorStop(args[2]);
  return { cx, cy, start, end };
};

const drawLinearBackground = (ctx: CanvasRenderingContext2D, size: number, value: string) => {
  const gradientInfo = parseLinearGradient(value);
  if (!gradientInfo) {
    ctx.fillStyle = value;
    ctx.fillRect(0, 0, size, size);
    return;
  }
  const angle = ((gradientInfo.angle - 90) * Math.PI) / 180;
  const half = size / 2;
  const dx = Math.cos(angle) * half;
  const dy = Math.sin(angle) * half;
  const gradient = ctx.createLinearGradient(half - dx, half - dy, half + dx, half + dy);
  gradient.addColorStop(0, gradientInfo.start);
  gradient.addColorStop(1, gradientInfo.end);
  ctx.fillStyle = gradient;
  ctx.fillRect(0, 0, size, size);
};

const drawRadialSpot = (
  ctx: CanvasRenderingContext2D,
  size: number,
  gradientValue: string,
  spotSizeValue: string,
  offsetXValue: string,
  offsetYValue: string
) => {
  const radial = parseRadialGradient(gradientValue);
  if (!radial) return;
  const radius = (parsePercent(spotSizeValue, 0.72) * size) / 2;
  const baseX = radial.cx * size;
  const baseY = radial.cy * size;
  const offsetX = parsePercent(offsetXValue, 0) * size;
  const offsetY = parsePercent(offsetYValue, 0) * size;
  const cx = baseX + offsetX;
  const cy = baseY + offsetY;
  const gradient = ctx.createRadialGradient(cx, cy, 0, cx, cy, radius);
  gradient.addColorStop(0, radial.start);
  gradient.addColorStop(1, radial.end);
  ctx.fillStyle = gradient;
  ctx.beginPath();
  ctx.arc(cx, cy, radius, 0, Math.PI * 2);
  ctx.fill();
};

const drawRing = (ctx: CanvasRenderingContext2D, size: number, ringColor: string, glowColor: string) => {
  const radius = size * 0.32;
  ctx.save();
  ctx.strokeStyle = ringColor;
  ctx.lineWidth = Math.max(1, size * 0.04);
  ctx.shadowColor = glowColor;
  ctx.shadowBlur = size * 0.18;
  ctx.beginPath();
  ctx.arc(size / 2, size / 2, radius, 0, Math.PI * 2);
  ctx.stroke();
  ctx.restore();
};

const renderCssAvatar = (avatar: string, size: number) => {
  const canvas = document.createElement('canvas');
  canvas.width = size;
  canvas.height = size;
  const ctx = canvas.getContext('2d');
  if (!ctx) return null;
  const vars = getAvatarVars(avatar);
  drawLinearBackground(ctx, size, vars['--avatar-bg'] ?? '#1f2937');
  drawRadialSpot(
    ctx,
    size,
    vars['--avatar-spot'] ?? 'radial-gradient(circle at 30% 30%, rgba(255,255,255,0.4), rgba(255,255,255,0))',
    vars['--avatar-spot-size'] ?? '72%',
    vars['--avatar-spot-x'] ?? '0%',
    vars['--avatar-spot-y'] ?? '0%'
  );
  drawRadialSpot(
    ctx,
    size,
    vars['--avatar-spot-2'] ?? 'radial-gradient(circle at 70% 70%, rgba(255,255,255,0.2), rgba(255,255,255,0))',
    vars['--avatar-spot-2-size'] ?? '48%',
    vars['--avatar-spot-2-x'] ?? '0%',
    vars['--avatar-spot-2-y'] ?? '0%'
  );
  drawRing(ctx, size, vars['--avatar-ring'] ?? 'rgba(255,255,255,0.4)', vars['--avatar-glow'] ?? 'rgba(255,255,255,0.2)');
  return canvas;
};

const renderImageAvatar = async (avatar: string, size: number) => {
  const url = await resolveAvatarUrl(avatar);
  if (!url) return null;
  const image = await new Promise<HTMLImageElement | null>((resolve) => {
    const img = new Image();
    img.crossOrigin = 'anonymous';
    img.onload = () => resolve(img);
    img.onerror = () => resolve(null);
    img.src = url;
  });
  if (!image) return null;
  const canvas = document.createElement('canvas');
  canvas.width = size;
  canvas.height = size;
  const ctx = canvas.getContext('2d');
  if (!ctx) return null;
  const scale = Math.max(size / image.width, size / image.height);
  const sw = size / scale;
  const sh = size / scale;
  const sx = (image.width - sw) / 2;
  const sy = (image.height - sh) / 2;
  ctx.drawImage(image, sx, sy, sw, sh, 0, 0, size, size);
  return canvas;
};

const canvasToPngBytes = async (canvas: HTMLCanvasElement) => {
  const blob = await new Promise<Blob | null>((resolve) => {
    canvas.toBlob((result) => resolve(result), 'image/png');
  });
  if (!blob) return null;
  const buffer = await blob.arrayBuffer();
  return Array.from(new Uint8Array(buffer));
};

/**
 * 生成托盘头像 PNG 字节。
 * 输入：头像字符串与目标尺寸（默认 64）。
 * 输出：PNG 字节数组或 null。
 */
export const renderAvatarPngBytes = async (avatar?: string | null, size = 64): Promise<number[] | null> => {
  if (typeof window === 'undefined') {
    return null;
  }
  const value = ensureAvatar(avatar);
  const canvas = isCssAvatar(value)
    ? renderCssAvatar(value, size)
    : await renderImageAvatar(value, size);
  if (!canvas) return null;
  return canvasToPngBytes(canvas);
};
