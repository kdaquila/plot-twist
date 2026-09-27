// Tick generation and label formatting (FR-006a, FR-009h). Date/time values are seconds
// since 1970 on a naive timeline, so all time formatting uses UTC (no zone conversion).
import { scaleLinear, scaleUtc } from "d3-scale";
import { utcDay, utcMinute, utcMonth, utcSecond, utcYear } from "d3-time";
import { utcFormat } from "d3-time-format";

export const PX_PER_TICK = 80;

export interface Tick {
  value: number;
  label: string;
}

/** Scientific notation when any non-zero |value| is ≥ 1e6 or < 1e-3. */
export function needsScientific(values: readonly number[]): boolean {
  return values.some((v) => v !== 0 && (Math.abs(v) >= 1e6 || Math.abs(v) < 1e-3));
}

function scientific(value: number, step: number): string {
  if (value === 0) return "0";
  const exponent = Math.floor(Math.log10(Math.abs(value)));
  const digits = Math.max(0, Math.min(6, exponent - Math.floor(Math.log10(Math.abs(step)))));
  const text = value.toExponential(digits);
  const [mantissa = "", power = ""] = text.split("e");
  const trimmed = mantissa.includes(".") ? mantissa.replace(/\.?0+$/, "") : mantissa;
  return `${trimmed}e${power.replace("+", "")}`;
}

export function numericTicks(min: number, max: number, lengthPx: number): Tick[] {
  const count = Math.max(2, Math.round(lengthPx / PX_PER_TICK));
  const scale = scaleLinear().domain([min, max]);
  const values = scale.ticks(count);
  if (needsScientific(values)) {
    const step = values.length > 1 ? Math.abs((values[1] ?? 0) - (values[0] ?? 0)) : 1;
    return values.map((value) => ({ value, label: scientific(value, step || 1) }));
  }
  const format = scale.tickFormat(count);
  return values.map((value) => ({ value, label: format(value) }));
}

export function timeTicks(minSeconds: number, maxSeconds: number, lengthPx: number): Tick[] {
  const count = Math.max(2, Math.round(lengthPx / PX_PER_TICK));
  const scale = scaleUtc().domain([new Date(minSeconds * 1000), new Date(maxSeconds * 1000)]);
  return scale.ticks(count).map((date) => ({
    value: date.getTime() / 1000,
    label: timeLabel(date),
  }));
}

const formatMillisecond = utcFormat(".%L");
const formatSecond = utcFormat(":%S");
const formatMinute = utcFormat("%H:%M");
const formatDay = utcFormat("%b %d");
const formatMonth = utcFormat("%b");
const formatYear = utcFormat("%Y");

/** The coarsest label that identifies the tick; day, month, and year boundaries give context. */
function timeLabel(date: Date): string {
  if (utcSecond(date) < date) return formatMillisecond(date);
  if (utcMinute(date) < date) return formatSecond(date);
  if (utcDay(date) < date) return formatMinute(date);
  if (utcMonth(date) < date) return formatDay(date);
  if (utcYear(date) < date) return formatMonth(date);
  return formatYear(date);
}

const fullTime = utcFormat("%Y-%m-%d %H:%M:%S.%L");

/** Full date-time for the hover readout. */
export function formatDateTime(seconds: number): string {
  return fullTime(new Date(seconds * 1000)).replace(/\.000$/, "");
}

/** Exact value for the hover readout. */
export function formatExact(value: number): string {
  return needsScientific([value]) ? value.toExponential(6) : String(Number(value.toPrecision(12)));
}
