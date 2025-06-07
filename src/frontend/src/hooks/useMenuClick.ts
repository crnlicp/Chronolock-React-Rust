import { useEffect } from 'react';
import { useLocation } from 'react-router';

export const useMenuClick = () => {
  const location = useLocation();

  useEffect(() => {
    const href = location.hash.replace(/^\//, '');
    if (href.startsWith('#')) {
      const id = href.slice(1);
      const targetElement = document.getElementById(id);
      if (targetElement) {
        targetElement.scrollIntoView({ behavior: 'smooth' });
      }
    }
  }, [location]);
};
