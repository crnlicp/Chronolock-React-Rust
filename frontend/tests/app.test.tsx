import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { App } from '../src/app/App';

describe('App', () => {
  it('renders the home page with navigation and sign-in', async () => {
    render(<App />);
    expect(await screen.findByRole('heading', { level: 1 })).toHaveTextContent(/message/i);
    expect(screen.getAllByRole('button', { name: 'Sign in' }).length).toBeGreaterThan(0);
    expect(screen.getAllByRole('link', { name: 'Create' }).length).toBeGreaterThan(0);
    expect(screen.getAllByRole('link', { name: 'Wallet' }).length).toBeGreaterThan(0);
  });
});
