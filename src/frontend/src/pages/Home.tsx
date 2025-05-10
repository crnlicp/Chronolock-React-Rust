import { Fragment } from 'react';
import { HeroSlider } from '../components/HeroSlider';
import { SectionsDevider } from '../components/SectionsDevider';
import { FunFacts } from '../components/FunFacts';
import { About } from '../components/About';
import { Collection } from '../components/Collection';
import { RoadMapSlider } from '../components/RoadMapSlider';
import { Faqs } from '../components/Faqs';

export const Home = () => {
  return (
    <Fragment>
      <HeroSlider />
      <SectionsDevider />
      <FunFacts />
      <SectionsDevider />
      <About />
      <SectionsDevider />
      <Collection />
      <SectionsDevider />
      <RoadMapSlider />
      <SectionsDevider />
      <Faqs />
    </Fragment>
  );
};
