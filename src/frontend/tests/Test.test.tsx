import { render, screen } from '@testing-library/react';
import { expect } from 'vitest';
import '@testing-library/jest-dom';

describe('Example Test', () => {
  it('renders a simple div', () => {
    render(<div>Hello World</div>);
    expect(screen.getByText('Hello World')).toBeInTheDocument();
  });
});
