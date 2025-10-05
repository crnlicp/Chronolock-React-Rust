import { Fragment, useEffect } from 'react';
import { HeroSlider } from '../components/HeroSlider';
import { SectionsDevider } from '../components/SectionsDevider';
import { FunFacts } from '../components/FunFacts';
import { About } from '../components/About';
import { Collection } from '../components/Collection';
import { RoadMapSlider } from '../components/RoadMapSlider';
import { Faqs } from '../components/Faqs';
import { dataBgImg, imgToSVG } from '../utils/utility';

export const Home = () => {
  useEffect(() => {
    dataBgImg();
    imgToSVG();
  }, []);

  const targetDate = new Date();
  targetDate.setHours(targetDate.getHours() + 19);

  return (
    <Fragment>
      <HeroSlider targetDate={targetDate} />
      <SectionsDevider />
      <FunFacts />
      <SectionsDevider />
      <About targetDate={targetDate} />
      <SectionsDevider />
      <Collection targetDate={targetDate} />
      <SectionsDevider />
      <RoadMapSlider />
      <SectionsDevider />
      <Faqs />
    </Fragment>
  );
};
