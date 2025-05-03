const Header = () => {
    return (
        <header id="header">
            <div className="header">
                <div className="header_in">
                    <div className="trigger_logo">
                        <div className="trigger" >
                            <span />
                        </div>
                        <div className="logo">
                            <a href="/">
                                <img src="/img/logo.png" alt="" />
                            </a>
                        </div>
                    </div>
                    <div className="nav" style={{ opacity: 1 }}>
                        <ul>
                            <li>
                                <a href="/#home" className="creative_link">Home</a>
                            </li>
                            <li>
                                <a href="/#about" className="creative_link">About</a>
                            </li>
                            <li>
                                <a href="/#collection" className="creative_link">Collection</a>
                            </li>
                            <li>
                                <a href="/#news" className="creative_link">Blog</a>
                            </li>
                            <li>
                                <a href="/#contact" className="creative_link">Contact</a>
                            </li>
                        </ul>
                    </div>
                    <div className="wallet">
                        <a
                            href="#"
                            onClick={(e) => {
                                e.preventDefault();

                            }}
                            className="metaportal_fn_button wallet_opener"
                        >
                            <span>Connect To Wallet</span>
                        </a>
                    </div>
                </div>
            </div>
        </header>
    )
}

export default Header
