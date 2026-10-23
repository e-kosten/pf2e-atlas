export const nonNegativeIntegerRule = (required: boolean) => ({
  validator: (_rule: unknown, value: number | null | undefined) => {
    if (value === null || value === undefined)
      return required
        ? Promise.reject(new Error("Enter a whole number."))
        : Promise.resolve();
    return Number.isSafeInteger(value) && value >= 0
      ? Promise.resolve()
      : Promise.reject(new Error("Enter a whole number, zero or greater."));
  },
});
