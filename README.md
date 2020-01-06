# Weather-Forecast
[![](https://img.shields.io/badge/license-GPL%20v3-green.svg)](https://github.com/jiayaoqijia/Weather-Forecast/blob/master/LICENSE)
[![](https://img.shields.io/github/last-commit/jiayaoqijia/Weather-Forecast)](https://github.com/jiayaoqijia/Weather-Forecast/)
[![](https://img.shields.io/github/repo-size/jiayaoqijia/Weather-Forecast)](https://github.com/jiayaoqijia/Weather-Forecast/)
[![](https://img.shields.io/github/issues/jiayaoqijia/Weather-Forecast)](https://github.com/jiayaoqijia/Weather-Forecast/issues)
[![](https://img.shields.io/github/issues-pr/jiayaoqijia/Weather-Forecast)](https://github.com/jiayaoqijia/Weather-Forecast/pulls)

A blockchain for weather forecast based on [Substrate](https://github.com/paritytech/substrate). We use off-chain worker to fetch data from [OpenWeather](https://openweathermap.org/) and automatically submit on-chain transactions of weather proposals to the chain. With an intuitive governance mechanism, a group of selected vote authorities will vote for and confirm the valid proposals through transactions. Meanwhile, they can also cast votes to kick out mis-behavior proposal/vote authorities, if the number of votes pass the threshold. 


## Available features

The current version has the following features implemented:

* Use off-chain worker to send HTTP request
* Parse JSON data using non-std JSON parser
* Fetch weather data from OpenWeather via off-chain worker
* Submit signed transaction in off-chain worker
* Sudo account adds/removes proposal/vote authorities
* Proposal authorities send on-chain transactions of weather proposals
* Vote authorities vote for valid weather proposals
* Vote authorities cast votes to kick out proposal/vote authorities
* Use PolkadotJS to fetch/parse data from the chain
* UI demonstrates the on-chain weather data via PolkadotJS

## How to build the chain:

 * Run `git clone https://github.com/jiayaoqijia/Weather-Forecast.git`.
 * Go into the `chain` folder and run:
 * Run 

    ```
    ./scripts/init.sh
    cargo build --release
    ./target/release/weather-forecast --dev
    ```

    The above process may take 30 minuites or so, depending on your hardware. This should start your node, and you should see blocks being created.
    
## How to build the Weather UI:

* Go into the `ui` folder and run:

    ```
    yarn 
    yarn build
    yarn start
    ```

    This should start a web server on `localhost:8080` where you can see the weather fetched from the chain as shown below.
<p align="center">
  <img src="https://github.com/jiayaoqijia/Weather-Forecast/blob/master/ui/preview.png" width=50%>
</p>
    
## How to build the PolkadotJS apps:

* Go into the `apps` folder and run:

    ```
    yarn 
    yarn run start
    ```

    This should start a web server on `localhost:3000` where you can interact with the chain as shown below.
[![](https://github.com/jiayaoqijia/Weather-Forecast/blob/master/docs/figures/check_weather.png?raw=true)]()

## How to build the tests:

* Go into the `scripts/tests` folder and run:

    ```
    yarn 
    yarn start
    ```

    This should use PolkadotJS apis to connect to/interact with the chain.

## How to interact with the chain:

* Please follow the steps in the [wiki](https://github.com/jiayaoqijia/Weather-Forecast/wiki/Steps-to-interact-with-the-chain).


## License

Weather Forecast is [GPL 3.0 licensed](LICENSE).