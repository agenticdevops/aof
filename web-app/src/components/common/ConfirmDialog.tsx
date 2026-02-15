import React from 'react'
import clsx from 'clsx'
import Modal from './Modal'
import Button from './Button'
import { ConfirmDialogProps } from '@/types'

export const ConfirmDialog: React.FC<ConfirmDialogProps> = ({
  isOpen,
  title,
  message,
  confirmText = 'Confirm',
  cancelText = 'Cancel',
  isDangerous = false,
  isLoading = false,
  onConfirm,
  onCancel,
}) => {
  return (
    <Modal isOpen={isOpen} onClose={onCancel} title={title} size="sm" showCloseButton={!isLoading}>
      <div className="space-y-4">
        <p className="text-gray-600 dark:text-gray-400">{message}</p>

        <div className="flex gap-3 justify-end pt-4">
          <Button variant="secondary" onClick={onCancel} disabled={isLoading}>
            {cancelText}
          </Button>
          <Button
            variant={isDangerous ? 'danger' : 'primary'}
            onClick={onConfirm}
            loading={isLoading}
            disabled={isLoading}
          >
            {confirmText}
          </Button>
        </div>
      </div>
    </Modal>
  )
}

export default ConfirmDialog
