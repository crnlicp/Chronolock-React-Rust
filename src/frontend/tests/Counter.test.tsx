import { render } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { Counter } from '../src/components/Counter';

describe('Counter Component', () => {
  it('renders with default props', () => {
    render(<Counter />);
    expect(document.body).toBeTruthy();
  });

  it('renders with custom props', () => {
    const { container } = render(<Counter end={100} decimals={2} />);

    expect(container.firstChild).toBeTruthy();
  });

  it('displays counter element', () => {
    const { container } = render(<Counter end={50} />);

    const counterElement = container.querySelector('.fn_cs_counter');
    expect(counterElement).toBeTruthy();
  });

  it('handles different end values', () => {
    const { container } = render(<Counter end={1000} decimals={1} />);

    expect(container.firstChild).toBeTruthy();

    // Check data attribute is set
    const counterElement = container.querySelector('.fn_cs_counter');
    expect(counterElement?.getAttribute('data-to')).toBe('1000');
  });
});
