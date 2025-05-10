import { NavLink } from 'react-router';

export const Footer = () => {
  return (
    <footer id="footer">
      <div className="container">
        <div className="footer">
          <div className="left_part">
            <p>Copyright 2025</p>
          </div>
          <div className="right_part">
            <ul>
              <li>
                <NavLink to="/policy">
                  <a className="creative_link">Privacy Policy</a>
                </NavLink>
              </li>
              <li>
                <NavLink to="/cookies">
                  <a className="creative_link">Cookies</a>
                </NavLink>
              </li>
              <li>
                <NavLink to="/terms-conditions">
                  <a className="creative_link">Terms &amp; Conditions</a>
                </NavLink>
              </li>
            </ul>
          </div>
        </div>
      </div>
    </footer>
  );
};
