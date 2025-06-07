// Header
export const stickyNav = () => {
  const header = document.querySelector('.header');
  window.addEventListener('scroll', () => {
    let offset = window.scrollY;
    if (offset > 20) {
      header?.classList.add('active');
    } else {
      header?.classList.remove('active');
    }
  });
};

export const dataBgImg = () => {
  let d = document.querySelectorAll('[data-bg-img]');
  for (let i = 0; i < d.length; i++) {
    const element = d[i] as HTMLElement;
    element.style.backgroundImage = `url(${element.getAttribute(
      'data-bg-img',
    )})`;
  }
};

export const imgToSVG = () => {
  document.querySelectorAll('img.fn__svg').forEach((el) => {
    const imgID = el.getAttribute('id');
    const imgClass = el.getAttribute('class');
    const imgURL = el.getAttribute('src');

    if (imgURL) {
      fetch(imgURL)
        .then((data) => data.text())
        .then((response) => {
          const parser = new DOMParser();
          const xmlDoc = parser.parseFromString(response, 'text/html');
          let svg = xmlDoc.querySelector('svg');

          setTimeout(() => {
            if (svg !== null) {
              if (typeof imgID !== 'undefined') {
                if (imgID) {
                  svg.setAttribute('id', imgID);
                }
              }

              if (typeof imgClass !== 'undefined') {
                svg.setAttribute('class', imgClass + ' replaced-svg');
              }

              svg.removeAttribute('xmlns:a');

              el.parentNode && el.parentNode.replaceChild(svg, el);
            }
          }, 500);
        });
    }
  });
};
