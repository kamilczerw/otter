export function formatCurrency(minorUnits: number, decimalPlaces: number = 2): string {
  const major = minorUnits / Math.pow(10, decimalPlaces)
  return new Intl.NumberFormat('pl-PL', {
    style: 'currency',
    currency: 'PLN',
    minimumFractionDigits: decimalPlaces,
    maximumFractionDigits: decimalPlaces,
  }).format(major)
}

export function parseCurrencyToMinor(value: string, decimalPlaces: number = 2): number {
  const num = parseFloat(value)
  if (isNaN(num)) return 0
  return Math.round(num * Math.pow(10, decimalPlaces))
}
