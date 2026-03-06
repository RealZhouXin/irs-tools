export function isEscapeKey(event: KeyboardEvent): boolean {
  return event.key === "Escape";
}

export function isSpaceKey(event: KeyboardEvent): boolean {
  return event.key === " " || event.code === "Space" || event.key === "Spacebar";
}

export function isInteractiveDialogTarget(target: EventTarget | null): boolean {
  if (!(target instanceof Element)) {
    return false;
  }

  return (
    target.closest(
      [
        "button",
        "[role='button']",
        "input",
        "textarea",
        "select",
        "a[href]",
        "[contenteditable='true']",
        "[data-dialog-shortcut-ignore='true']",
      ].join(","),
    ) !== null
  );
}
