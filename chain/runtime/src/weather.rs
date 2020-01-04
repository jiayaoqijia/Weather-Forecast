/// A runtime module for Weather Forecast

/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/master/frame/example/src/lib.rs
use rstd::{prelude::*, vec::Vec};
use support::{debug, decl_event, decl_module, decl_storage, dispatch};
use sp_runtime::traits::Hash;
use system::{ensure_signed, ensure_root, offchain};
use codec::{Decode, Encode};
use sp_runtime::{offchain::http, DispatchError};
use runtime_io::offchain::random_seed;
use simple_json::{ self, json::JsonValue };
use num_traits::float::FloatCore;

use primitives::crypto::KeyTypeId;
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"wea!");

pub mod crypto {
	use super::KEY_TYPE;
	use sp_runtime::app_crypto::{app_crypto, sr25519};
	app_crypto!(sr25519, KEY_TYPE);
}

#[derive(Debug, Encode, Decode, Clone, PartialEq, Eq, Default)]
// #[cfg_attr(feature = "std", derive(Debug))]
pub struct Weather {
	time: u64,
	city: Vec<u8>,
	main: Vec<u8>,
	description: Vec<u8>,
	icon: Vec<u8>,
// All data are multiplied with 1000
	temp: u32,
	humidity: u32,
	wind: u32,
	clouds: u32,
	sunrise: u64,
    sunset: u64
}

/// The module's configuration trait.
pub trait Trait: system::Trait + timestamp::Trait {
	// TODO: Add other types and constants required configure this module.

	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

	/// The overarching event type.
	type Call: From<Call<Self>>;

	/// Transaction submitter.
	type SubmitTransaction: offchain::SubmitSignedTransaction<Self, <Self as Trait>::Call>;
}

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as WeatherForecast {

		// Accounts able to send weather proposals.
		ProposalAuthorities get(fn proposal_authorities): Vec<T::AccountId>;
		// Accounts able to send votes.
		VoteAuthorities get(fn vote_authorities): Vec<T::AccountId>;
		// Threshold of votes to confirm proposals etc.
		VoteThreshold get(fn vote_threshold): u64;

		// Basic information of a proposal and the owner.
		Proposals: map T::Hash => Weather;
		ProposalOwner: map T::Hash => T::AccountId;

		// Basic information of all the proposals.
		AllProposalsArray: map u64 => T::Hash;
		AllProposalsIndex: map T::Hash => u64;
		AllProposalsCount get(fn all_proposals_count): u64;

		// Basic information of the owner's proposals.
		OwnedProposalsArray: map (T::AccountId, u64) => T::Hash;
		OwnedProposalsIndex: map T::Hash => (T::AccountId, u64);
		OwnedProposalsCount: map T::AccountId => u64;

		// Basic information of all the confirmed proposals.
		ProposalConfirmation: map T::Hash => Vec<T::AccountId>;
		AllConfirmedProposalsArray: map u64 => T::Hash;
		AllConfirmedProposalsIndex: map T::Hash => u64;
		AllConfirmedProposalsCount get(fn all_confirmed_proposals_count): u64;

		// The reputation of proposal/vote authorities.
		ProposalAuthorityReputation: map T::AccountId => Vec<T::AccountId>;
		VoteAuthorityReputation: map T::AccountId => Vec<T::AccountId>;

		Nonce get(fn nonce): u64;
	}
}

// The module's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		fn deposit_event() = default;

		/// Add accounts as proposal authorities, which submits weather proposals via offchain worker.
		pub fn add_proposal_authority(origin, who: T::AccountId) -> dispatch::DispatchResult {
			// This is protected by a root-call (e.g. through governance like `sudo`).
			debug::info!("add_proposal_authority who is {:?}", &who);
	
			let _me = ensure_root(origin)?;
			debug::info!("add_proposal_authority is {:?}", Self::proposal_authorities());
	
			if !Self::is_proposal_authority(&who) {
				debug::info!("Is_proposal_authority ProposalAuthorities is {:?}", Self::proposal_authorities());
				<Self as Store>::ProposalAuthorities::mutate(|l| l.push(who.clone()));
			}

			Self::deposit_event(RawEvent::ProposalAuthoritiesAdded(who));
			Ok(())
		}

		/// Deletes proposal authorities.
		pub fn delete_proposal_authority(origin, who: T::AccountId) -> dispatch::DispatchResult {
			// This is protected by a root-call (e.g. through governance like `sudo`).
			let _me = ensure_root(origin)?;
	
			if Self::is_proposal_authority(&who) {
				Self::remove_proposal_authority(&who);
			}

			Self::deposit_event(RawEvent::ProposalAuthoritiesDeleted(who));
			Ok(())
		}

		/// Add accounts as vote authorities,
		/// which vote for weather proposals via offchain worker.
		pub fn add_vote_authority(origin, who: T::AccountId) -> dispatch::DispatchResult {
			// This is protected by a root-call (e.g. through governance like `sudo`).
			let _me = ensure_root(origin)?;
			debug::info!("add_vote_authority is {:?}", Self::vote_authorities());
	
			if !Self::is_vote_authority(&who) {
				debug::info!("is_vote_authority VoteAuthorities is {:?}", Self::vote_authorities());
				<Self as Store>::VoteAuthorities::mutate(|l| l.push(who.clone()));
			}

			Self::deposit_event(RawEvent::VoteAuthoritiesAdded(who));
			Ok(())
		}

		/// Delete vote authorities.
		pub fn delete_vote_authority(origin, who: T::AccountId) -> dispatch::DispatchResult {
			// This is protected by a root-call (e.g. through governance like `sudo`).
			let _me = ensure_root(origin)?;
	
			if Self::is_vote_authority(&who) {
				Self::remove_vote_authority(&who);
			}

			Self::deposit_event(RawEvent::VoteAuthoritiesDeleted(who));
			Ok(())
		}

		/// Set the threshold of # votes for a weather proposal to be confirmed.
		pub fn set_vote_threshold(origin, threshold: u64) -> dispatch::DispatchResult {
			// This is protected by a root-call (e.g. through governance like `sudo`).
			let _me = ensure_root(origin)?;
			debug::info!("set_vote_threshold is {:?}", Self::vote_threshold());
	
			<Self as Store>::VoteThreshold::put(&threshold);
			debug::info!("set_vote_threshold VoteThreshold is {:?}", Self::vote_threshold());

			Self::deposit_event(RawEvent::VoteThresholdSet(threshold));
			Ok(())
		}

		fn on_initialize(_now: T::BlockNumber) {
			// At the beginning of each block execution, system triggers all
			// `on_initialize` functions, which allows us to set up some temporary state or - like
		}

		/// Submit a proposal of weather forecast.
		pub fn submit_weather_proposal(origin, weather: Weather, hash: T::Hash) -> dispatch::DispatchResult {
			let sender = ensure_signed(origin)?;

			// Check if the sender is a proposal authority.
			if !Self::is_proposal_authority(&sender) {
				// TODO: return error
				debug::info!("{:?} is not in the proposal authority list.", &sender);
				// return Ok(());
				return Err(DispatchError::Other("submit_weather_proposal: sender is not a proposal authority."));
			}

			// Add the weather proposal and proposal's owner.
			<Self as Store>::Proposals::insert(&hash, &weather);
			debug::info!("Proposals is {:?}", <Self as Store>::Proposals::get(&hash));
			<Self as Store>::ProposalOwner::insert(&hash, &sender);
			debug::info!("ProposalOwner is {:?}", <Self as Store>::ProposalOwner::get(&hash));

			// Update the basic information of all the proposals.
			<Self as Store>::AllProposalsArray::insert(<Self as Store>::AllProposalsCount::get(), &hash);
			debug::info!("AllProposalsArray is {:?}", <Self as Store>::AllProposalsArray::get(<Self as Store>::AllProposalsCount::get()));
			<Self as Store>::AllProposalsIndex::insert(&hash, <Self as Store>::AllProposalsCount::get());
			<Self as Store>::AllProposalsCount::mutate(|i| *i += 1);
			debug::info!("AllProposalsCount is {:?}", <Self as Store>::AllProposalsCount::get());
			
			// Update the basic information of the proposer's proposals.
			<Self as Store>::OwnedProposalsArray::insert((&sender, <Self as Store>::OwnedProposalsCount::get(&sender)), &hash);
			let index = <Self as Store>::OwnedProposalsCount::get(&sender);
			<Self as Store>::OwnedProposalsIndex::insert(&hash, (&sender, index));
			<Self as Store>::OwnedProposalsCount::mutate(&sender, |i| *i += 1);

			debug::info!("OwnedProposalsCount is {:?}", <Self as Store>::OwnedProposalsCount::get(&sender));
			
			// Emit an event to indicate that the weather proposal has been submitted.
			Self::deposit_event(RawEvent::WeatherProposalSubmitted(sender, hash, weather));

			Ok(())
		}

		/// Submit a vote for a weather proposal.
		pub fn submit_weather_vote(origin, hash: T::Hash) -> dispatch::DispatchResult {

			let sender = ensure_signed(origin)?;

			// Check if the sender is a vote authority. 
			if !Self::is_vote_authority(&sender) {
				// TODO: return error
				debug::info!("{:?} is not in the vote authority list.", &sender);
				// return Ok(());
				return Err(DispatchError::Other("submit_weather_vote: sender is not a vote authority."));
			}

			// Check if the sender has already voted for this proposal.
			if <Self as Store>::ProposalConfirmation::get(&hash).iter().find(|&j| j == &sender).is_some() {
				debug::info!("{:?} already votes for this proposal.", &sender);
				// return Ok(());
				return Err(DispatchError::Other("submit_weather_vote: sender has voted."));
			}
			else {
				<Self as Store>::ProposalConfirmation::mutate(&hash, |i| {
					i.push(sender.clone());
					debug::info!("ProposalConfirmation new vote is {:?}", &sender);
				});
			}
	
			debug::info!("ProposalConfirmation is {:?}", <Self as Store>::ProposalConfirmation::get(&hash));
			
			// Check if the proposal receives a sufficient number of votes. If yes, it will be confirmed.
			if <Self as Store>::ProposalConfirmation::get(&hash).len() as u64 == Self::vote_threshold() {
				<Self as Store>::AllConfirmedProposalsArray::insert(Self::all_confirmed_proposals_count(), &hash);
				<Self as Store>::AllConfirmedProposalsIndex::insert( &hash, Self::all_confirmed_proposals_count());
				<Self as Store>::AllConfirmedProposalsCount::mutate(|i| *i += 1);
				debug::info!("AllConfirmedProposalsCount is {:?}", Self::all_confirmed_proposals_count());
			}
			
			// Emit an event for the weather vote.
			Self::deposit_event(RawEvent::WeatherVoteSubmitted(sender, hash));
			Ok(())
		}

		/// Cast a vote to kick out a proposal authority.
		pub fn submit_proposal_authority_reputation(origin, who: T::AccountId) -> dispatch::DispatchResult {

			let sender = ensure_signed(origin)?;

			// Check if the sender is a vote authority.
			if !Self::is_vote_authority(&sender) {
				// TODO: return error
				debug::info!("{:?} is not in the vote authority list.", &sender);
				// return Ok(());
				return Err(DispatchError::Other("submit_proposal_authority_reputation: sender is not a vote authority."));
			}
	
			// Check if the proposer is a valid proposal authority.
			if !Self::is_proposal_authority(&who) {
				debug::info!("{:?} is not in the proposal authority list.", &who);
				// return Ok(());
				return Err(DispatchError::Other("submit_proposal_authority_reputation: proposer is not a proposal authority."));
			}

			// Check if the sender has already cast the same vote.
			if <Self as Store>::ProposalAuthorityReputation::get(&who).iter().find(|&j| j == &sender).is_some() {
				debug::info!("{:?} already votes for this proposal.", &sender);
				// return Ok(());
				return Err(DispatchError::Other("submit_proposal_authority_reputation: sender has voted."));
			}
			else {
				<Self as Store>::ProposalAuthorityReputation::mutate(&who, |i| {
					i.push(sender.clone());
					debug::info!("ProposalAuthorityReputation new vote is {:?}", &sender);
				});
			}
	
			debug::info!("ProposalAuthorityReputation is {:?}", <Self as Store>::ProposalAuthorityReputation::get(&who));
			
			// Check if the proposer receives a sufficient number of votes. If yes, it will be kicked out of the proposal authority list.
			if <Self as Store>::ProposalAuthorityReputation::get(&who).len() as u64 == Self::vote_threshold() {
				Self::remove_proposal_authority(&who);
				<Self as Store>::ProposalAuthorityReputation::remove(&who);
				debug::info!("ProposalAuthority {:?} is deleted", &who);
			}
			
			// Emit an event for the reputation vote.
			Self::deposit_event(RawEvent::ProposalAuthorityReputationVoteSubmitted(sender, who));
			Ok(())
		}

		/// Cast a vote to kick out a vote authority.
		pub fn submit_vote_authority_reputation(origin, who: T::AccountId) -> dispatch::DispatchResult {

			let sender = ensure_signed(origin)?;

			// Check if the sender is a vote authority.
			if !Self::is_vote_authority(&sender) {
				// TODO: return error
				debug::info!("{:?} is not in the vote authority list.", &sender);
				// return Ok(());
				return Err(DispatchError::Other("submit_vote_authority_reputation: sender is not a vote authority."));
			}
	
			// Check if the user is a vote authority.
			if !Self::is_vote_authority(&who) {
				debug::info!("{:?} is not in the vote authority list.", &who);
				// return Ok(());
				return Err(DispatchError::Other("submit_vote_authority_reputation: user is not a vote authority."));
			}

			// Check if the sender has already cast the same vote.
			if <Self as Store>::VoteAuthorityReputation::get(&who).iter().find(|&j| j == &sender).is_some() {
				debug::info!("{:?} already votes for this proposal.", &sender);
				// return Ok(());
				return Err(DispatchError::Other("submit_vote_authority_reputation: sender has voted."));
			}
			else {
				<Self as Store>::VoteAuthorityReputation::mutate(&who, |i| {
					i.push(sender.clone());
					debug::info!("VoteAuthorityReputation new vote is {:?}", &sender);
				});
			}
	
			debug::info!("VoteAuthorityReputation is {:?}", <Self as Store>::VoteAuthorityReputation::get(&who));
			
			// Check if the user receives a sufficient number of votes. If yes, it will be kicked out of the vote authority list.
			if <Self as Store>::VoteAuthorityReputation::get(&who).len() as u64 == Self::vote_threshold() {
				Self::remove_vote_authority(&who);
				<Self as Store>::VoteAuthorityReputation::remove(&who);
				debug::info!("VoteAuthority {:?} is deleted", &who);
			}
			
			// Emit an event for the reputation vote.
			Self::deposit_event(RawEvent::VoteAuthorityReputationVoteSubmitted(sender, who));
			Ok(())
		}


		fn offchain_worker(block_number: T::BlockNumber) {

			debug::info!("Offchain worker starts!");
			debug::info!("Current block number is: {:?}", block_number);

			let weather = match Self::fetch_weather() {
				Ok(weather) => {
				  debug::info!("Got weather: {:?}", weather);
				  weather
				},
				_ => {
				  debug::error!("Error fetching weather.");
				  return
				}
			};

			Nonce::mutate(|n| {
				*n += 1
			});

			// Generate a random hash as an ID for the weather proposal.
			let nonce = Nonce::get();
			debug::info!("Nonce is {:?}", &nonce);

			let random_num = random_seed();
			debug::info!("random_seed is {:?}", &random_num);
			let random_hash = (&random_num, &nonce)
				.using_encoded(T::Hashing::hash);
			
			debug::info!("random_hash: {:?}", &random_hash);

			debug::info!("Weather is {:?}", &weather);

			Self::submit_weather_proposal_on_chain(weather, random_hash);
		}
	}
}

impl<T: Trait> Module<T> {

	/// Helper that confirms whether the given `AccountId` can send `proposal` transactions.
	fn is_proposal_authority(who: &T::AccountId) -> bool {
		let result = Self::proposal_authorities().into_iter().find(|i| i == who).is_some();
		debug::info!("is_proposal_authority result is {:?}", &result);
		result
	}

	/// Helper that removes the proposal authority of the given `AccountId`.
	fn remove_proposal_authority(who: &T::AccountId){
		<Self as Store>::ProposalAuthorities::mutate(|l| {
			let position = Self::proposal_authorities().into_iter().position(|i| i == *who).unwrap();
			debug::info!("remove_proposal_authority position is {:?}", &position);
			l.remove(position);
		});
	}

	/// Helper that confirms whether the given `AccountId` can send `vote` transactions
	fn is_vote_authority(who: &T::AccountId) -> bool {
		let result = Self::vote_authorities().into_iter().find(|i| i == who).is_some();
		debug::info!("is_vote_authority result is {:?}", &result);
		result
	}

	/// Helper that removes the vote authority of the given `AccountId`.
	fn remove_vote_authority(who: &T::AccountId){
		<Self as Store>::VoteAuthorities::mutate(|l| {
			let position = Self::vote_authorities().into_iter().position(|i| i == *who).unwrap();
			debug::info!("remove_vote_authority position is {:?}", &position);
			l.remove(position);
		});
	}

	/// Helper that parses the JSON data to weather using simplejson.
	fn parse_weather_json(json_val: JsonValue) -> Result<Weather, http::Error> {
		// Sample weather data in JSON format.
		// {"coord":{"lon":-0.13,"lat":51.51},"weather":[{"id":803,"main":"Clouds","description":"broken clouds","icon":"04n"}],"base":"stations","main":{"temp":277.69,"feels_like":272.73,"temp_min":275.93,"temp_max":279.15,"pressure":1033,"humidity":81},"visibility":10000,"wind":{"speed":4.6,"deg":240},"clouds":{"all":51},"dt":1578119710,"sys":{"type":1,"id":1414,"country":"GB","sunrise":1578125144,"sunset":1578153856},"timezone":0,"id":2643743,"name":"London","cod":200}
		let main = json_val.get_object()[1].1.get_array()[0].get_object()[1].1.get_string();
		let description = json_val.get_object()[1].1.get_array()[0].get_object()[2].1.get_string();
		let icon = json_val.get_object()[1].1.get_array()[0].get_object()[3].1.get_string();
		debug::info!("main, description, icon is {:?}, {:?}, {:?}", &main, &description, &icon);

		let temp_f64 = json_val.get_object()[3].1.get_object()[0].1.get_number_f64();
		let temp = (temp_f64 * 1000.0).round() as u32;
		let humidity_f64 = json_val.get_object()[3].1.get_object()[5].1.get_number_f64();
		let humidity = (humidity_f64 * 1000.0).round() as u32;
		debug::info!("temp, humidity is {:?}, {:?}", &temp, &humidity);

		let wind_f64 = json_val.get_object()[5].1.get_object()[0].1.get_number_f64();
		let wind = (wind_f64 * 1000.0).round() as u32;
		let clouds_f64 = json_val.get_object()[6].1.get_object()[0].1.get_number_f64();
		let clouds = (clouds_f64 * 1000.0).round() as u32;

		let time = json_val.get_object()[7].1.get_number_f64() as u64;

		let sunrise = json_val.get_object()[8].1.get_object()[3].1.get_number_f64() as u64;
		let sunset = json_val.get_object()[8].1.get_object()[4].1.get_number_f64() as u64;

		let city = json_val.get_object()[11].1.get_string();

		debug::info!("time: {:?} \n 
		city: {:?} \n 
		main: {:?} \n 
		description: {:?} \n 
		icon: {:?} \n 
		temp: {:?} \n 
		humidity: {:?} \n 
		wind: {:?} \n 
		clouds: {:?} \n 
		sunrise: {:?} \n 
		sunset: {:?} \n 
		", &time, &city, &main, &description, &icon, &temp, &humidity, 
		&wind, &clouds, &sunrise, &sunset);

		let main_bytes = json_val.get_object()[1].1.get_array()[0].get_object()[1].1.get_bytes();
		let description_bytes = json_val.get_object()[1].1.get_array()[0].get_object()[2].1.get_bytes();
		let icon_bytes = json_val.get_object()[1].1.get_array()[0].get_object()[3].1.get_bytes();
		let city_bytes = json_val.get_object()[11].1.get_bytes();
		debug::info!("main_bytes, description_bytes, icon_bytes, city_bytes is {:?}, {:?}, {:?}, {:?}", &main_bytes, &description_bytes, 
		&icon_bytes, &city_bytes);

		let weather = Weather {
			time: time,
			city: city_bytes,
			main: main_bytes,
			description: description_bytes,
			icon: icon_bytes,
			temp: temp,
			humidity: humidity,
			wind: wind,
			clouds: clouds,
			sunrise: sunrise,
			sunset: sunset
		};
		debug::info!("Weather is {:?}", &weather);
		Ok(weather)

	}
	
	/// Helper that fetches weather data in JSON format using off-chain worker's HTTP request.
	fn fetch_weather() -> Result<Weather, http::Error> {
		let pending = http::Request::get(
        "https://api.openweathermap.org/data/2.5/weather?q=London,uk&APPID=47c69406d636336123cbef9721328177"
      ).send().map_err(|_| http::Error::IoError)?;

		let response = pending.wait()?;
		if response.code != 200 {
			debug::warn!("Unexpected status code: {}", response.code);
			return Err(http::Error::Unknown);
		}

		let body = response.body().collect::<Vec<u8>>();
		debug::info!("HTTP response: {:?}", core::str::from_utf8(&body).ok());
		let json_val: JsonValue = simple_json::parse_json(
			&core::str::from_utf8(&body).unwrap())
			.unwrap();

		let weather = Self::parse_weather_json(json_val).unwrap();

		Ok(weather)
	}

	/// Helper that submits an on-chain transaction for the weather forecast.
	fn submit_weather_proposal_on_chain(weather: Weather, hash: T::Hash) {
		use system::offchain::SubmitSignedTransaction;
		let call = Call::submit_weather_proposal(weather, hash);
		let res = T::SubmitTransaction::submit_signed(call);

		if res.is_empty() {
			debug::error!("No local accounts found.");
		} else {
			debug::info!("Sent transactions from: {:?}", res);
		}
	}
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId,
		Hash = <T as system::Trait>::Hash
	{
		// Event to add a proposal authority.
		ProposalAuthoritiesAdded(AccountId),
		// Event to remove a proposal authority.
		ProposalAuthoritiesDeleted(AccountId),
		// Event to add a vote authority.
		VoteAuthoritiesAdded(AccountId),
		// Event to remove a vote authority.
		VoteAuthoritiesDeleted(AccountId),
		// Event to set a threshold for votes.
		VoteThresholdSet(u64),
		// Event to submit a weather proposal.
		WeatherProposalSubmitted(AccountId, Hash, Weather),
		// Event to submit a weather vote.
		WeatherVoteSubmitted(AccountId, Hash),
		// Event to cast a vote to kick out a proposal authority.
		ProposalAuthorityReputationVoteSubmitted(AccountId, AccountId),
		// Event to cast a vote to kick out a vote authority.
		VoteAuthorityReputationVoteSubmitted(AccountId, AccountId),
	}
);

/// TODO: 
/// CI/CD
/// Generic options to provide data for any input
/// Better erro handling
/// Bump to the latest version
/// Update README and wiki to walk through how to use this module.
/// Add local tests
/// Capture deposit events
/// Modify frame/system/src/offchain.rs; src/service.rs
/// Support forecast of multiple days, cities, etc.
/// UI allows users to select cities
/// Chain supports to get data (city) from users and provision results.

/// tests for this module
#[cfg(test)]
mod tests {
	use super::*;

	use primitives::H256;
	use sp_runtime::{
		testing::Header,
		traits::{BlakeTwo256, IdentityLookup},
		Perbill,
	};
	use support::{assert_ok, impl_outer_origin, parameter_types, weights::Weight};

	impl_outer_origin! {
		pub enum Origin for Test {}
	}

	// For testing the module, we construct most of a mock runtime. This means
	// first constructing a configuration type (`Test`) which `impl`s each of the
	// configuration traits of modules we want to use.
	#[derive(Clone, Eq, PartialEq)]
	pub struct Test;
	parameter_types! {
		pub const BlockHashCount: u64 = 250;
		pub const MaximumBlockWeight: Weight = 1024;
		pub const MaximumBlockLength: u32 = 2 * 1024;
		pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
	}
	impl system::Trait for Test {
		type Origin = Origin;
		type Call = ();
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type AccountId = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type Event = ();
		type BlockHashCount = BlockHashCount;
		type MaximumBlockWeight = MaximumBlockWeight;
		type MaximumBlockLength = MaximumBlockLength;
		type AvailableBlockRatio = AvailableBlockRatio;
		type Version = ();
	}
	impl Trait for Test {
		type Event = ();
	}
	type WeatherForecast = Module<Test>;

	// This function basically just builds a genesis storage key/value store according to
	// our desired mockup.
	fn new_test_ext() -> runtime_io::TestExternalities {
		system::GenesisConfig::default()
			.build_storage::<Test>()
			.unwrap()
			.into()
	}

	#[test]
	fn it_works_for_default_value() {
		new_test_ext().execute_with(|| {
			// Just a dummy test for the dummy funtion `do_something`
			// calling the `do_something` function with a value 42
			// assert_ok!(WeatherForecast::do_something(Origin::signed(1), 42));
			// asserting that the stored value is equal to what we stored
			// assert_eq!(WeatherForecast::something(), Some(42));
		});
	}
}
