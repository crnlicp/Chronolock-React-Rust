import { BrowserRouter, Route, Routes } from 'react-router';
import { ActorContextProvider } from './ActorContextProvider';
import { Footer } from './components/Footer';
import { Header } from './components/header/Header';
import { NotFound } from './components/NotFound';
import { ChronolockDetail } from './pages/ChronolockDetail';
import { Collection } from './pages/Collection';
import { Cookies } from './pages/Cookies';
import { Create } from './pages/Create';
import { Home } from './pages/Home';
import { Policy } from './pages/Policy';
import { TermsConditions } from './pages/TermsConditions';

const App = () => {
  return (
    <ActorContextProvider>
      <BrowserRouter>
        <Header />
        <div className="body_container">
          <Routes>
            <Route path="/" element={<Home />} />
            <Route path="/chronolock/:id" element={<ChronolockDetail />} />
            <Route path="/create" element={<Create />} />
            <Route path="/collection" element={<Collection />} />
            <Route path="/policy" element={<Policy />} />
            <Route path="/cookies" element={<Cookies />} />
            <Route path="/terms-conditions" element={<TermsConditions />} />
            <Route path="*" element={<NotFound />} />
          </Routes>
        </div>
        <Footer />
      </BrowserRouter>
    </ActorContextProvider>
  );
};

export default App;
