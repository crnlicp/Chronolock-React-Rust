import SwiperCore from 'swiper';
import 'swiper/swiper-bundle.css';

const roadmapStep = (
  mySwiper: any,
  step: Element | null,
  widthParts: number,
) => {
  var breakpoint = parseInt(mySwiper.width);
  var viewBox;
  switch (breakpoint) {
    case 1400:
      viewBox = 4;
      break;
    case 1200:
      viewBox = 3;
      break;
    case 1040:
      viewBox = 4;
      break;
    case 768:
      viewBox = 1;
      break;
    case 100:
      viewBox = 1;
      break;
    default:
      viewBox = 4;
  }

  if (step instanceof HTMLElement) {
    step.style.width = (mySwiper.activeIndex + viewBox) * widthParts + '%';
  }
};
export const roadMapProps = {
  loop: false,
  speed: 1500,
  autoplay: {
    delay: 5000,
    disableOnInteraction: false,
  },
  slidesPerView: 3,
  spaceBetween: 30,
  direction: 'horizontal' as 'horizontal' | 'vertical',
  loopAdditionalSlides: 10,
  watchSlidesProgress: true,
  breakpoints: {
    100: {
      slidesPerView: 1,
    },
    768: {
      slidesPerView: 1,
    },
    1040: {
      slidesPerView: 2,
    },
    1200: {
      slidesPerView: 3,
    },
    1400: {
      slidesPerView: 4,
    },
  },

  onSwiper: function (mySwiper: SwiperCore) {
    var slidersCount = mySwiper.params.loop
      ? mySwiper.slides.length - 2
      : mySwiper.slides.length;
    var widthParts = 100 / slidersCount;
    var step = document.querySelector('.fn_cs_roadmap .step_in');
    roadmapStep(mySwiper, step, widthParts);
  },

  onSlideChange: function (mySwiper: SwiperCore) {
    var slidersCount = mySwiper.params.loop
      ? mySwiper.slides.length - 2
      : mySwiper.slides.length;
    var widthParts = 100 / slidersCount;
    var step = document.querySelector('.fn_cs_roadmap .step_in');
    roadmapStep(mySwiper, step, widthParts);
  },
};

// export const Hero4Slider = {
//   loop: true,
//   speed: 1000,
//   autoplay: {
//     delay: 3000,
//     disableOnInteraction: false,
//   },
//   slidesPerView: 'auto',
//   spaceBetween: 50,
//   direction: 'horizontal',
//   loopAdditionalSlides: 10,
//   watchSlidesProgress: true,
// };

// export const hero6Slider = {
//   loop: true,
//   speed: 1500,
//   autoplay: {
//     delay: 5000,
//     disableOnInteraction: false,
//   },
//   navigation: {
//     nextEl: '.next',
//     prevEl: '.prev',
//   },
//   slidesPerView: 1,
//   loopAdditionalSlides: 10,
//   watchSlidesProgress: true,
// };
