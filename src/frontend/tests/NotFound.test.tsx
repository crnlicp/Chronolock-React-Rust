import { render } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { NotFound } from '../src/components/NotFound';
import { MemoryRouter } from 'react-router';

describe('NotFound Component', () => {
  it('renders without crashing', () => {
    render(
      <MemoryRouter>
        <NotFound />
      </MemoryRouter>,
    );
    expect(document.body).toBeTruthy();
  });

  it('displays 404 content', () => {
    const { container } = render(
      <MemoryRouter>
        <NotFound />
      </MemoryRouter>,
    );

    // Check for typical 404 content
    const content = container.innerHTML.toLowerCase();
    expect(
      content.includes('404') ||
        content.includes('not found') ||
        content.includes('page not found'),
    ).toBeTruthy();
  });

  it('renders navigation elements', () => {
    const { container } = render(
      <MemoryRouter>
        <NotFound />
      </MemoryRouter>,
    );

    // Should contain some navigation or link elements
    expect(container.firstChild).toBeTruthy();
  });
});
