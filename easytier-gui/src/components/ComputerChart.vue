<script setup lang="ts">
import { computed, ref } from 'vue'

export interface ComputerChartSeries {
  values: Array<number | null>
  color: string
  label?: string
}

const props = withDefaults(defineProps<{
  series: ComputerChartSeries[]
  max: number
  name: string
  format: (value: number) => string
  area?: boolean
  xStartLabel?: string
  sampleIntervalSecs?: number
  capacity?: number
  locale?: string
}>(), {
  area: false,
  xStartLabel: '5 分钟前',
  sampleIntervalSecs: 5,
  capacity: 60,
  locale: 'cn',
})

const root = ref<HTMLElement | null>(null)
const hoverIndex = ref<number | null>(null)

const rawSampleCount = computed(() =>
  Math.max(0, ...props.series.map(series => series.values.length)),
)

const pointCount = computed(() =>
  Math.max(2, Math.min(props.capacity, rawSampleCount.value)),
)

const gridY = [4, 50, 96]

function xOf(index: number) {
  return index / (pointCount.value - 1) * 100
}

function yOf(value: number) {
  const safeMax = Math.max(props.max, Number.EPSILON)
  const ratio = Math.min(1, Math.max(0, value / safeMax))
  return 4 + (1 - ratio) * 92
}

function segmentsFor(values: Array<number | null>) {
  const value = values.length === 1 ? values[0] : null
  const normalized = value == null || !Number.isFinite(value)
    ? values
    : [value, value]
  const offset = pointCount.value - normalized.length
  const segments: Array<{ path: string, first: number, last: number }> = []
  let path = ''
  let first = -1
  let last = -1

  const flush = () => {
    if (first >= 0)
      segments.push({ path, first, last })
    path = ''
    first = -1
    last = -1
  }

  normalized.forEach((value, index) => {
    if (value == null || !Number.isFinite(value)) {
      flush()
      return
    }
    const slot = offset + index
    path += `${first < 0 ? 'M' : 'L'}${xOf(slot).toFixed(2)} ${yOf(value).toFixed(2)}`
    if (first < 0)
      first = slot
    last = slot
  })
  flush()
  return segments
}

function areaFor(values: Array<number | null>) {
  return segmentsFor(values)
    .map(segment =>
      `${segment.path}L${xOf(segment.last).toFixed(2)} 96L${xOf(segment.first).toFixed(2)} 96Z`,
    )
    .join(' ')
}

function pathFor(values: Array<number | null>) {
  return segmentsFor(values).map(segment => segment.path).join(' ')
}

function valueAt(series: ComputerChartSeries, slot: number): number | null {
  if (series.values.length === 1 && slot >= 0 && slot < pointCount.value) {
    const only = series.values[0]
    return only == null || !Number.isFinite(only) ? null : only
  }
  const index = slot - (pointCount.value - series.values.length)
  if (index < 0 || index >= series.values.length)
    return null
  const value = series.values[index]
  return value == null || !Number.isFinite(value) ? null : value
}

function hoverValue(series: ComputerChartSeries) {
  return hoverIndex.value == null ? null : valueAt(series, hoverIndex.value)
}

function text(cn: string, en: string) {
  return props.locale === 'cn' ? cn : en
}

function updateFromPointer(event: PointerEvent) {
  const element = root.value
  if (!element)
    return
  const rect = element.getBoundingClientRect()
  if (rect.width <= 0)
    return
  const ratio = Math.min(1, Math.max(0, (event.clientX - rect.left) / rect.width))
  hoverIndex.value = Math.round(ratio * (pointCount.value - 1))
}

function hideHover() {
  hoverIndex.value = null
}

function onKeydown(event: KeyboardEvent) {
  const steps: Record<string, number> = {
    ArrowLeft: -1,
    ArrowRight: 1,
    Home: -Number.MAX_SAFE_INTEGER,
    End: Number.MAX_SAFE_INTEGER,
  }
  const step = steps[event.key]
  if (step == null)
    return
  event.preventDefault()
  const current = hoverIndex.value ?? pointCount.value - 1
  hoverIndex.value = Math.min(
    pointCount.value - 1,
    Math.max(0, current + step),
  )
}

function agoLabel(index: number) {
  if (rawSampleCount.value <= 1)
    return text('现在', 'now')
  const seconds = (pointCount.value - 1 - index) * props.sampleIntervalSecs
  if (seconds <= 0)
    return text('现在', 'now')
  if (seconds < 60)
    return text(`${seconds} 秒前`, `${seconds}s ago`)
  const minutes = Math.round(seconds / 60)
  if (minutes < 60)
    return text(`${minutes} 分钟前`, `${minutes}m ago`)
  const hours = Math.floor(minutes / 60)
  const rest = minutes % 60
  return rest
    ? text(`${hours} 小时 ${rest} 分前`, `${hours}h ${rest}m ago`)
    : text(`${hours} 小时前`, `${hours}h ago`)
}

const tooltipLeft = computed(() => {
  if (hoverIndex.value == null)
    return ''
  const x = xOf(hoverIndex.value)
  return x > 55 ? '' : `calc(${x}% + 10px)`
})

const tooltipRight = computed(() => {
  if (hoverIndex.value == null)
    return ''
  const x = xOf(hoverIndex.value)
  return x > 55 ? `calc(${100 - x}% + 10px)` : ''
})

const ariaLabel = computed(() => {
  const current = props.series.map((series) => {
    const value = series.values.at(-1)
    const prefix = series.label ? `${series.label} ` : ''
    return `${prefix}${value == null ? text('无数据', 'no data') : props.format(value)}`
  }).join(text('，', ', '))
  return text(
    `${props.name}，最近 ${agoLabel(0)}趋势，当前 ${current}。使用左右方向键查看历史数值`,
    `${props.name}, trend ending ${agoLabel(0)}, currently ${current}. Use the arrow keys to inspect history`,
  )
})
</script>

<template>
  <div class="computer-chart">
    <div
      ref="root"
      class="computer-chart-plot"
      role="img"
      tabindex="0"
      :aria-label="ariaLabel"
      @pointermove="updateFromPointer"
      @pointerdown="updateFromPointer"
      @pointerleave="hideHover"
      @blur="hideHover"
      @keydown="onKeydown"
    >
      <svg viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">
        <line
          v-for="y in gridY"
          :key="y"
          class="computer-chart-grid"
          x1="0"
          x2="100"
          :y1="y"
          :y2="y"
          vector-effect="non-scaling-stroke"
        />
        <path
          v-for="(item, seriesIndex) in series"
          v-show="area"
          :key="`area-${seriesIndex}`"
          class="computer-chart-area"
          :d="areaFor(item.values)"
          :style="{ fill: item.color }"
        />
        <path
          v-for="(item, seriesIndex) in series"
          :key="`line-${seriesIndex}`"
          class="computer-chart-line"
          :d="pathFor(item.values)"
          :style="{ stroke: item.color }"
          vector-effect="non-scaling-stroke"
        />
      </svg>

      <span class="computer-chart-y computer-chart-y-top">
        {{ format(max) }}
      </span>
      <span class="computer-chart-y computer-chart-y-mid">
        {{ format(max / 2) }}
      </span>

      <template v-if="hoverIndex != null">
        <span
          class="computer-chart-crosshair"
          :style="{ left: `${xOf(hoverIndex)}%` }"
        />
        <span
          v-for="(item, seriesIndex) in series"
          :key="`dot-${seriesIndex}`"
          class="computer-chart-dot"
          :class="{ 'computer-chart-dot-hidden': hoverValue(item) == null }"
          :style="{
            left: `${xOf(hoverIndex)}%`,
            top: `${hoverValue(item) == null ? 0 : yOf(hoverValue(item) as number)}%`,
            background: item.color,
          }"
        />

        <div
          class="computer-chart-tooltip"
          :style="{ left: tooltipLeft, right: tooltipRight }"
        >
          <div
            v-for="(item, seriesIndex) in series"
            :key="`tooltip-${seriesIndex}`"
            class="computer-chart-tooltip-row"
          >
            <span class="computer-chart-tooltip-swatch" :style="{ background: item.color }" />
            <strong>{{ hoverValue(item) == null ? text('无数据', 'No data') : format(hoverValue(item) as number) }}</strong>
            <span v-if="item.label">{{ item.label }}</span>
          </div>
          <div class="computer-chart-tooltip-time">
            {{ agoLabel(hoverIndex) }}
          </div>
        </div>
      </template>
    </div>

    <div class="computer-chart-axis" aria-hidden="true">
      <span>{{ xStartLabel }}</span>
      <span>{{ text('现在', 'Now') }}</span>
    </div>
  </div>
</template>

<style scoped>
.computer-chart {
  min-width: 0;
  width: 100%;
}

.computer-chart-plot {
  position: relative;
  width: 100%;
  min-width: 0;
  height: 96px;
  border-radius: 8px;
  touch-action: pan-y;
  cursor: crosshair;
  outline: none;
}

.computer-chart-plot:focus-visible {
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent, #2a78d6) 55%, transparent);
}

.computer-chart-plot svg {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  overflow: visible;
}

.computer-chart-grid {
  stroke: var(--chart-grid, rgba(68, 71, 78, 0.14));
  stroke-width: 1;
  fill: none;
}

.computer-chart-line {
  fill: none;
  stroke-width: 2;
  stroke-linejoin: round;
  stroke-linecap: round;
}

.computer-chart-area {
  stroke: none;
  opacity: 0.14;
}

.computer-chart-y {
  position: absolute;
  left: 2px;
  font-size: 10px;
  line-height: 12px;
  color: var(--muted);
  opacity: 0.8;
  pointer-events: none;
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
  text-shadow:
    0 0 2px var(--panel, #fff),
    0 0 4px var(--panel, #fff);
}

.computer-chart-y-top {
  top: calc(4% + 2px);
}

.computer-chart-y-mid {
  top: calc(50% + 2px);
}

.computer-chart-crosshair {
  position: absolute;
  top: 0;
  bottom: 0;
  width: 1px;
  background: var(--muted);
  opacity: 0.5;
  pointer-events: none;
}

.computer-chart-dot {
  position: absolute;
  width: 10px;
  height: 10px;
  margin: -5px 0 0 -5px;
  border-radius: 50%;
  border: 2px solid var(--panel, #fff);
  pointer-events: none;
}

.computer-chart-dot-hidden {
  display: none;
}

.computer-chart-tooltip {
  position: absolute;
  top: 4px;
  z-index: 3;
  display: none;
  min-width: 96px;
  max-width: calc(100% - 16px);
  padding: 6px 10px;
  border-radius: 8px;
  background: var(--inverse-surface, #2e3036);
  color: var(--inverse-on-surface, #f0f0f7);
  box-shadow:
    0 2px 6px 2px rgba(0, 0, 0, 0.12),
    0 1px 2px rgba(0, 0, 0, 0.2);
  font-size: 12px;
  line-height: 16px;
  white-space: nowrap;
  pointer-events: none;
}

.computer-chart-plot:where(:hover, :focus-visible) .computer-chart-tooltip,
.computer-chart-plot:has(.computer-chart-crosshair) .computer-chart-tooltip {
  display: block;
}

.computer-chart-tooltip-row {
  display: flex;
  align-items: center;
  gap: 6px;
}

.computer-chart-tooltip-swatch {
  width: 10px;
  height: 2px;
  border-radius: 1px;
  flex: none;
}

.computer-chart-tooltip strong {
  font-size: 13px;
  font-weight: 500;
  font-variant-numeric: tabular-nums;
}

.computer-chart-tooltip-time {
  margin-top: 2px;
  opacity: 0.72;
}

.computer-chart-axis {
  display: flex;
  justify-content: space-between;
  gap: 8px;
  margin-top: 4px;
  color: var(--muted);
  font-size: 11px;
  opacity: 0.8;
}

@media (max-width: 600px) {
  .computer-chart-plot {
    height: 96px;
  }

  .computer-chart-tooltip {
    max-width: calc(100vw - 72px);
    overflow: hidden;
    text-overflow: ellipsis;
  }
}

@media (prefers-reduced-motion: reduce) {
  .computer-chart-plot {
    scroll-behavior: auto;
  }
}
</style>
