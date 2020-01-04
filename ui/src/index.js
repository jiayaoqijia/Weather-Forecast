import css from './style.css';
import API_KEY from './config.js';
import Handler from './modules/handler.js';
import UI from './modules/ui.js';
import { ApiPromise, WsProvider } from '@polkadot/api';

let metric = true;
let weatherRes = null;

$(document).ready(function(){
  UI.updateWeather(metric);
  // Handler.fetchChainWeather(metric);
  fetchChainState(metric).catch(console.error);
  // console.log( 'weather is: ' + JSON.stringify(weather));

  // if(navigator.geolocation) {
  //   navigator.geolocation.getCurrentPosition(success, error);
  // } else {
  //   error();
  // }

  $("#fahrenheit-now, #celsius-now").on("click", function(){
    if($(this).hasClass("degree-inactive")){
      $(this).removeClass("degree-inactive");
      $(".degree-active").removeClass("degree-active").addClass("degree-inactive");
      $(this).addClass("degree-active");

      if($(this).is("#fahrenheit-now")) {
        metric = false;
      } else {
        metric = true;
      }

      UI.updateWeather(metric);
    }
  });
});

function error() {
  alert("Sorry, no position available.");
}

function success(position) {
  handleRequest(position.coords.latitude, position.coords.longitude);
}

function handleRequest(latitude, longitude){
  Handler.fetchCurrentWeather(latitude, longitude, metric);
  Handler.fetchForecast(latitude, longitude, metric);
}

export default metric;

// async function fetchChainWeather(metric) {
//   let weather = await fetchChainWeather().catch(console.error);
//   console.log( 'weather is: ' + JSON.stringify(weather));
//   // UI.setCurrentWeather(weather, metric);
// }

async function fetchChainState(metric) {
  // Initialise the provider to connect to the local node
  const wsProvider = new WsProvider("ws://127.0.0.1:9944");
  // const wsProvider = new WsProvider("ws://10.1.1.54:9944");
  // const wsProvider = new WsProvider("wss://cc3-5.kusama.network");

  // Create the API and wait until ready
  const api = await ApiPromise.create({ 
      provider: wsProvider,
      types: {
          Weather: {

              time: "u64",
              city: "Vec<u8>",
              main: "Vec<u8>",
              description: "Vec<u8>",
              icon: "Vec<u8>",
          // All data are multiplied with 1000
              temp: "u32",
              humidity: "u32",
              wind: "u32",
              clouds: "u32",
              sunrise: "u64",
              sunset: "u64"
          }
        }
   });

  
  const now = await api.query.timestamp.now();
  console.log( 'date is: ' + now );

  // const { magicNumber,metadata } = await api.rpc.state.getMetadata();
  // console.log( 'Magic number: ' + magicNumber );
  // console.log( 'Metadata: ' + metadata.raw );

  // console.log(api.genesisHash.toHex());

  // Retrieve the chain & node information information via rpc calls
  const [chain, nodeName, nodeVersion] = await Promise.all([
    api.rpc.system.chain(),
    api.rpc.system.name(),
    api.rpc.system.version()
  ]);

  console.log(
    `You are connected to chain ${chain} using ${nodeName} v${nodeVersion}`
  );

  const [proposalAuthorities] = await Promise.all([
    api.query.weatherForecast.proposalAuthorities()
  ]);
  console.log( 'proposalAuthorities is: ' + proposalAuthorities);

  const [allProposalsCount, allConfirmedProposalsCount] = await Promise.all([
    api.query.weatherForecast.allProposalsCount(),
    api.query.weatherForecast.allConfirmedProposalsCount(),
  ]);
  console.log( 'allProposalsCount is: ' + allProposalsCount);
  console.log( 'allConfirmedProposalsCount is: ' + allConfirmedProposalsCount);

  let index = allProposalsCount - 1;
  let confirmIndex = allConfirmedProposalsCount - 1;

  const hash = await api.query.weatherForecast.allProposalsArray(index);
  const confirmHash = await api.query.weatherForecast.allConfirmedProposalsArray(confirmIndex);
  console.log( 'hash is: ' + hash);
  console.log( 'confirmHash is: ' + confirmHash);

  const weather = await api.query.weatherForecast.proposals(hash);
  const confirmWeather = await api.query.weatherForecast.proposals(confirmHash);

  console.log( 'weather is: ' + weather);
  console.log( 'confirmWeather is: ' + confirmWeather);

  let weatherObj = JSON.parse(weather);
  let confirmWeatherObj = JSON.parse(confirmWeather);

  weatherObj.city = hex2a(weatherObj.city);
  weatherObj.main = hex2a(weatherObj.main);
  weatherObj.icon = hex2a(weatherObj.icon);
  weatherObj.description = hex2a(weatherObj.description);
  // Kelvin to Fahrenheit
  weatherObj.temp = ((weatherObj.temp / 1000 - 273.15) * 1.8) + 32;

  confirmWeatherObj.city = hex2a(confirmWeatherObj.city);
  confirmWeatherObj.main = hex2a(confirmWeatherObj.main);
  confirmWeatherObj.icon = hex2a(confirmWeatherObj.icon);
  confirmWeatherObj.description = hex2a(confirmWeatherObj.description);
  // Kelvin to Fahrenheit
  confirmWeatherObj.temp = ((confirmWeatherObj.temp / 1000 - 273.15) * 1.8) + 32;
  
  console.log( 'weatherObj is: ' + JSON.stringify(weatherObj));
  console.log( 'confirmWeatherObj is: ' + JSON.stringify(confirmWeatherObj));

  let des = weatherObj.description;
  let confirmDes = confirmWeatherObj.description;
  console.log( 'des is: ' + des);
  console.log( 'confirmDes is: ' + confirmDes);

  let weatherRes = {
    city: weatherObj.city,
    cloudiness: weatherObj.clouds / 1000,
    country: "GB",
    desc: weatherObj.description,
    humidity: weatherObj.humidity / 1000,
    sunrise: weatherObj.sunrise,
    sunset: weatherObj.sunset,
    temp: weatherObj.temp,
    type: weatherObj.icon,
    wind: weatherObj.wind / 1000
  };
  console.log( 'weatherRes is: ' + JSON.stringify(weatherRes));
  // metric = true;
  // UI.updateWeather(metric);
  Handler.fetchWeatherState(weatherRes, metric);
  // return weatherRes;
}

function hex2a(hexx) {
  var hex = hexx.toString();//force conversion
  var str = '';
  for (var i = 0; (i < hex.length && hex.substr(i, 2) !== '00'); i += 2)
      str += String.fromCharCode(parseInt(hex.substr(i, 2), 16));
  return str;
}