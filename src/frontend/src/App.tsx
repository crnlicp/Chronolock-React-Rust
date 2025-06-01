import { BrowserRouter, Route, Routes } from 'react-router';
import { Home } from './pages/Home';
import { Chronolock } from './pages/Chronolock';
import { Create } from './pages/Create';
import { Collection } from './pages/Collection';
import { Header } from './components/header/Header';
import { Footer } from './components/Footer';
import { NotFound } from './components/NotFound';

const App = () => {
  return (
    <BrowserRouter>
      <Header />
      <div className="body_container">
        <Routes>
          <Route path="/" element={<Home />} />
          <Route path="/chronolock/:id" element={<Chronolock />} />
          <Route path="/create" element={<Create />} />
          <Route path="/collection" element={<Collection />} />
          <Route path="*" element={<NotFound />} />
        </Routes>
      </div>
      <Footer />
    </BrowserRouter>
  );
};

export default App;
