"use client";

import { useEffect, useRef } from "react";

export default function ConfirmDialog({
  title,
  description,
  confirmText = "Confirm",
  cancelText = "Cancel",
  onConfirm,
  onCancel,
  isDanger = false,
  isOpen = false,
  isSubmitting = false
}) {
  const dialogRef = useRef(null);

  useEffect(() => {
    const dialog = dialogRef.current;
    if (!dialog) return;

    if (isOpen && !dialog.open) {
      dialog.showModal();
    }

    if (!isOpen && dialog.open) {
      dialog.close();
    }
  }, [isOpen]);

  if (!isOpen) {
    return null;
  }

  return (
    <dialog
      className="confirm-dialog"
      ref={dialogRef}
      onClose={onCancel}
      onCancel={(event) => {
        event.preventDefault();
        onCancel();
      }}
    >
      <div className="confirm-dialog__content">
        <h3>{title}</h3>
        <p>{description}</p>
        <div className="confirm-dialog__actions">
          <button type="button" className="button-muted" onClick={onCancel} disabled={isSubmitting}>
            {cancelText}
          </button>
          <button
            type="button"
            className={isDanger ? "button-danger" : "button-primary"}
            onClick={onConfirm}
            disabled={isSubmitting}
          >
            {confirmText}
          </button>
        </div>
      </div>
    </dialog>
  );
}
