import { render } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import Clock from '../src/components/Clock';

describe('Clock Component', () => {
  it('renders without crashing', () => {
    const futureDate = new Date(Date.now() + 10000); // 10 seconds in future
    render(<Clock targetDate={futureDate} />);
    expect(document.body).toBeTruthy();
  });

  it('displays countdown elements', () => {
    const futureDate = new Date(Date.now() + 86400000); // 1 day in future
    const { container } = render(<Clock targetDate={futureDate} />);

    // Check that time units are rendered
    expect(container.innerHTML).toContain('days');
    expect(container.innerHTML).toContain('hours');
    expect(container.innerHTML).toContain('minutes');
    expect(container.innerHTML).toContain('seconds');
  });

  it('handles past dates', () => {
    const pastDate = new Date(Date.now() - 86400000); // 1 day ago
    const { container } = render(<Clock targetDate={pastDate} />);

    // Should render without errors even with past date
    expect(container.firstChild).toBeTruthy();
  });
});
