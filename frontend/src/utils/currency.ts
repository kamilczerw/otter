export function formatCurrency(minorUnits: number, decimalPlaces: number = 2): string {
  const major = minorUnits / Math.pow(10, decimalPlaces)
  return major.toFixed(decimalPlaces)
}

export function parseCurrencyToMinor(value: string, decimalPlaces: number = 2): number {
  const num = parseFloat(value)
  if (isNaN(num)) return 0
  return Math.round(num * Math.pow(10, decimalPlaces))
}
