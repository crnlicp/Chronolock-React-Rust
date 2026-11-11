import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router';
import { describe, expect, it } from 'vitest';
import { ClockLock } from '../src/components/clock-lock/ClockLock';
import { NotFoundPage } from '../src/pages/NotFound';

describe('ClockLock', () => {
  it('shows the padlock with a countdown while locked', () => {
    const { container } = render(<ClockLock target={new Date(Date.now() + 86_400_000)} />);
    expect(screen.getByRole('img', { name: /Locked until/ })).toBeInTheDocument();
    expect(container.querySelector('.clock-analog')).not.toBeNull();
    expect(container.textContent).toMatch(/Until Unlock/);
  });

  it('shows the open padlock without a clock once unlocked', () => {
    const { container } = render(<ClockLock target={new Date()} unlocked />);
    expect(screen.getByRole('img', { name: 'Unlocked' })).toBeInTheDocument();
    expect(container.querySelector('.clock-analog')).toBeNull();
  });
});

describe('NotFoundPage', () => {
  it('links home', () => {
    render(
      <MemoryRouter>
        <NotFoundPage />
      </MemoryRouter>,
    );
    expect(screen.getByText('404')).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Go home' })).toHaveAttribute('href', '/');
  });
});
