import type { Meta, StoryObj } from '@storybook/react'
import { Badge } from './Badge'

const meta: Meta<typeof Badge> = {
  component: Badge,
  title: 'Components/Common/Badge',
  tags: ['autodocs'],
  argTypes: {
    variant: {
      control: 'select',
      options: ['success', 'error', 'warning', 'info', 'neutral'],
    },
    size: {
      control: 'select',
      options: ['sm', 'md'],
    },
  },
}

export default meta
type Story = StoryObj<typeof meta>

export const Success: Story = {
  args: {
    children: 'Success',
    variant: 'success',
  },
}

export const Error: Story = {
  args: {
    children: 'Error',
    variant: 'error',
  },
}

export const Warning: Story = {
  args: {
    children: 'Warning',
    variant: 'warning',
  },
}

export const Info: Story = {
  args: {
    children: 'Info',
    variant: 'info',
  },
}

export const Neutral: Story = {
  args: {
    children: 'Neutral',
    variant: 'neutral',
  },
}

export const Small: Story = {
  args: {
    children: 'Small Badge',
    size: 'sm',
  },
}

export const Medium: Story = {
  args: {
    children: 'Medium Badge',
    size: 'md',
  },
}

export const WithIcon: Story = {
  args: {
    children: 'Connected',
    variant: 'success',
    icon: '✓',
  },
}
