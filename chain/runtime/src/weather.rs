/// A runtime module template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references

/// TODO: Add timestamp (Moment) (removed)
/// Parse JSON data; (done)
/// Put governance on proposal nodes; (done)
/// Filter nodes who can send proposals; (done)
/// CI/CD; 
/// Generic options to provide data for any input; 
/// Parsing data and code from input;
/// Offchain worker can only parse 248 transactions; (done)
/// Erro handling
/// Bump to the latest version (done)
/// Add deposit events (done)
/// Capture deposit events
/// Modify frame/system/src/offchain.rs; src/service.rs
/// Support forecast of multiple days, cities, etc.
/// UI allows users to select cities.
/// Chain supports to get data (city) from users and provision results.
/// Add readme for chain, ui and steps to operate the chain.


/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/master/frame/example/src/lib.rs
use support::{debug, decl_event, decl_module, decl_storage, dispatch, dispatch::DispatchError};
use sp_runtime::traits::Hash;
use system::{ensure_signed, ensure_root, offchain};
use rstd::vec::Vec;
use codec::{Decode, Encode};
use sp_runtime::offchain::http;
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
// pub struct Weather<Moment>{
pub struct Weather {
	// FIXME: Couldn't use Moment
	// time: Moment,
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
		// Just a dummy storage item.
		// Here we are declaring a StorageValue, `Something` as a Option<u32>
		// `get(fn something)` is the default getter which returns either the stored `u32` or `None` if nothing stored
		// Something get(fn something): Option<u32>;
		// Prices get(fn prices): Vec<u32>;
		// Proposals; 
		ProposalAuthorities get(fn proposal_authorities): Vec<T::AccountId>;
		VoteAuthorities get(fn vote_authorities): Vec<T::AccountId>;
		VoteThreshold get(fn vote_threshold): u64;

		Proposals: map T::Hash => Weather;
		ProposalOwner: map T::Hash => T::AccountId;

		AllProposalsArray: map u64 => T::Hash;
		AllProposalsIndex: map T::Hash => u64;
		AllProposalsCount get(fn all_proposals_count): u64;

		OwnedProposalsArray: map (T::AccountId, u64) => T::Hash;
		OwnedProposalsIndex: map T::Hash => (T::AccountId, u64);
		OwnedProposalsCount: map T::AccountId => u64;


		ProposalConfirmation: map T::Hash => Vec<T::AccountId>;
		AllConfirmedProposalsArray: map u64 => T::Hash;
		AllConfirmedProposalsIndex: map T::Hash => u64;
		AllConfirmedProposalsCount get(fn all_confirmed_proposals_count): u64;

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
		// this is needed only if you are using events in your module
		fn deposit_event() = default;

		// Just a dummy entry point.
		// function that can be called by the external world as an extrinsics call
		// takes a parameter of the type `AccountId`, stores it and emits an event
		// pub fn do_something(origin, something: u32) -> dispatch::DispatchResult {
		// 	// TODO: You only need this if you want to check it was signed.
		// 	let who = ensure_signed(origin)?;

		// 	// TODO: Code to execute when something calls this.
		// 	// For example: the following line stores the passed in u32 in the storage
		// 	Something::put(something);


		// 	// here we are raising the Something event
		// 	Self::deposit_event(RawEvent::SomethingStored(something, who));
		// 	Ok(())
		// }

		pub fn add_proposal_authority(origin, who: T::AccountId) -> dispatch::DispatchResult {
			// In practice this should be a bit cleverer, but for this example it is enough
			// that this is protected by a root-call (e.g. through governance like `sudo`).
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

		pub fn delete_proposal_authority(origin, who: T::AccountId) -> dispatch::DispatchResult {
			// In practice this should be a bit cleverer, but for this example it is enough
			// that this is protected by a root-call (e.g. through governance like `sudo`).
			let _me = ensure_root(origin)?;
	
			if Self::is_proposal_authority(&who) {
				Self::remove_proposal_authority(&who);
			}

			Self::deposit_event(RawEvent::ProposalAuthoritiesDeleted(who));
			Ok(())
		}

		pub fn add_vote_authority(origin, who: T::AccountId) -> dispatch::DispatchResult {
			// In practice this should be a bit cleverer, but for this example it is enough
			// that this is protected by a root-call (e.g. through governance like `sudo`).
			debug::info!("add_vote_authority who is {:?}", &who);
	
			let _me = ensure_root(origin)?;
			debug::info!("add_vote_authority is {:?}", Self::vote_authorities());
	
			if !Self::is_vote_authority(&who) {
				debug::info!("is_vote_authority VoteAuthorities is {:?}", Self::vote_authorities());
				<Self as Store>::VoteAuthorities::mutate(|l| l.push(who.clone()));
			}

			Self::deposit_event(RawEvent::VoteAuthoritiesAdded(who));
			Ok(())
		}

		pub fn delete_vote_authority(origin, who: T::AccountId) -> dispatch::DispatchResult {
			// In practice this should be a bit cleverer, but for this example it is enough
			// that this is protected by a root-call (e.g. through governance like `sudo`).
			let _me = ensure_root(origin)?;
	
			if Self::is_vote_authority(&who) {
				Self::remove_vote_authority(&who);
			}

			Self::deposit_event(RawEvent::VoteAuthoritiesDeleted(who));
			Ok(())
		}

		pub fn set_vote_threshold(origin, threshold: u64) -> dispatch::DispatchResult {
			// In practice this should be a bit cleverer, but for this example it is enough
			// that this is protected by a root-call (e.g. through governance like `sudo`).
			// Check if it's from root account
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
			// in this case - clean up other states
			// Nonce::put(0);
		  }

		
		pub fn submit_weather_proposal(origin, weather: Weather, hash: T::Hash) -> dispatch::DispatchResult {
			// let who = ensure_signed(origin)?;
			// FIXME: check if the sender is in proposalauthoritylist.

			let sender = ensure_signed(origin)?;

			if !Self::is_proposal_authority(&sender) {
				// TODO: return error
				debug::info!("{:?} is not in the proposal authority list.", &sender);
				return Ok(());
			}

			
			<Self as Store>::Proposals::insert(&hash, &weather);
			debug::info!("11 Proposals is {:?}", <Self as Store>::Proposals::get(&hash));
			<Self as Store>::ProposalOwner::insert(&hash, &sender);
			debug::info!("11 owner_of is {:?}", <Self as Store>::ProposalOwner::get(&hash));
			// Proposals get(fn proposal): map T::Hash => Weather;
			// ProposalOwner get(fn owner_of): map T::Hash => T::AccountId;

			<Self as Store>::AllProposalsArray::insert(<Self as Store>::AllProposalsCount::get(), &hash);
			debug::info!("11 AllProposalsArray is {:?}", <Self as Store>::AllProposalsArray::get(<Self as Store>::AllProposalsCount::get()));
			<Self as Store>::AllProposalsIndex::insert(&hash, <Self as Store>::AllProposalsCount::get());
			<Self as Store>::AllProposalsCount::mutate(|i| *i += 1);

			debug::info!("11 all_proposals_count is {:?}", <Self as Store>::AllProposalsCount::get());
			// AllProposalsArray get(fn proposal_by_index): map u64 => T::Hash;
			// AllProposalsIndex: map T::Hash => u64;
			// AllProposalsCount get(fn all_proposals_count): u64;
			

			<Self as Store>::OwnedProposalsArray::insert((&sender, <Self as Store>::OwnedProposalsCount::get(&sender)), &hash);
			let index = <Self as Store>::OwnedProposalsCount::get(&sender);
			<Self as Store>::OwnedProposalsIndex::insert(&hash, (&sender, index));
			<Self as Store>::OwnedProposalsCount::mutate(&sender, |i| *i += 1);

			debug::info!("11 OwnedProposalsCount is {:?}", <Self as Store>::OwnedProposalsCount::get(&sender));
			
			// OwnedProposalsArray get(fn proposal_of_owner_by_index): map (T::AccountId, u64) => T::Hash;
			// OwnedProposalsIndex: map T::Hash => (T::AccountId, u64);
			// OwnedProposalsCount get(fn owned_proposal_count): map T::AccountId => u64;
			

			// ProposalConfirmation get(fn confirmation_of): map T::Hash => Vec<T::AccountId>;
			// AllConfirmedProposalsArray get(fn confirmed_proposal_by_index): map u64 => T::Hash;
			// AllConfirmedProposalsIndex: map T::Hash => u64;
			// AllConfirmedProposalsCount get(fn all_confirmed_proposals_count): u64;
			

			// Nonce get(fn nonce): u64;

			// debug::info!("Nonce: {:?}", &sender);
			// let nonce = <Nonce>::get();
			// debug::info!("Nonce: {:?}", &nonce);
            // let random_hash = (random_seed(), &sender, &nonce)
				// .using_encoded(T::Hashing::hash);
				
			// let weather = Weather {
			// 	time: b"1485789600".to_vec(),
			// 	country: b"UK".to_vec(),
			// 	city: b"London".to_vec(),
			// 	description: b"Foggy".to_vec(),
			// 	temp: b"180".to_vec()
			// };
	

			// <Proposals<T>>::insert(&random_hash, Some(weather));
			// <ProposalOwner<T>>::insert(&random_hash, &sender);

			// <AllProposalsArray<T>>::insert(&nonce, &random_hash);
			// <AllProposalsIndex<T>>::insert(&random_hash, &nonce);
			// <AllProposalsCount>::put(&nonce);

			// <Nonce>::mutate(|n| *n += 1);

			// debug::info!("random_hash: {:?}", &random_hash);
			// debug::info!("sender: {:?}", &sender);
			// debug::info!("Nonce: {:?}", &nonce);

			// Self::deposit_event(RawEvent::NewWeather(sender, random_hash));

			// debug::info!("Adding to the average: {}", price);
			// let average = Prices::mutate(|prices| {
			// 	const MAX_LEN: usize = 64;

			// 	if prices.len() < MAX_LEN {
			// 		prices.push(price);
			// 	} else {
			// 		prices[price as usize % MAX_LEN] = price;
			// 	}

			// 	// TODO Whatchout for overflows
			// 	prices.iter().sum::<u32>() / prices.len() as u32
			// });
			// debug::info!("Current average price is: {}", average);

			// here we are raising the Something event
			// Self::deposit_event(RawEvent::NewWeather(weather, who));
			// AllProposalsCount::put(123);
			// let nonce = Nonce::get();

			// debug::info!("11 Nonce is {:?}", &nonce);

			// let random_num = hash;
			// debug::info!("11 random_seed is {:?}", &random_num);
			// let random_hash = (&random_num, &nonce)
			// 	.using_encoded(T::Hashing::hash);
			
			// <AllProposalsIndex<T>>::insert(&random_hash, &nonce);

			// debug::info!("11 random_hash: {:?}", &random_hash);

			// let weather = Weather {
			// 	time: b"1485789600".to_vec(),
			// 	country: b"UK".to_vec(),
			// 	city: b"London".to_vec(),
			// 	description: b"Foggy".to_vec(),
			// 	temp: b"180".to_vec()
			// };

			// <Proposals<T>>::insert(&random_hash, &weather);
			// // <ProposalOwner<T>>::insert(&random_hash, &sender);
			// debug::info!("11 Weather is {:?}", &weather);

			// Nonce::mutate(|n| {
			// 	*n += 1
			// 	// match n.as_mut() {
			// 	// 	Some(x) => *x += 1,
			// 	// 	None => Nonce::put(1),
			// 	// }
			// });
			// debug::info!("11 Nonce after txn is {:?}", Nonce::get());
			Self::deposit_event(RawEvent::WeatherProposalSubmitted(sender, hash, weather));

			Ok(())
		}

		pub fn submit_weather_vote(origin, hash: T::Hash) -> dispatch::DispatchResult {
			// let who = ensure_signed(origin)?;

			let sender = ensure_signed(origin)?;

			if !Self::is_vote_authority(&sender) {
				// TODO: return error
				debug::info!("{:?} is not in the vote authority list.", &sender);
				return Ok(());
			}

			if <Self as Store>::ProposalConfirmation::get(&hash).iter().find(|&j| j == &sender).is_some() {
				debug::info!("{:?} already votes for this proposal.", &sender);
				return Ok(());
			}
			else {
				<Self as Store>::ProposalConfirmation::mutate(&hash, |i| {
					// if !i.iter().find(|&j| j == &sender).is_some() {
					i.push(sender.clone());
					debug::info!("ProposalConfirmation new vote is {:?}", &sender);
					// }
				});
			}
	
			debug::info!("ProposalConfirmation is {:?}", <Self as Store>::ProposalConfirmation::get(&hash));
			
			// FIXME: check if AllConfirmedProposalsIndex or hash is duplicated (done)

			if <Self as Store>::ProposalConfirmation::get(&hash).len() as u64 == Self::vote_threshold() {
				<Self as Store>::AllConfirmedProposalsArray::insert(Self::all_confirmed_proposals_count(), &hash);
				<Self as Store>::AllConfirmedProposalsIndex::insert( &hash, Self::all_confirmed_proposals_count());
				<Self as Store>::AllConfirmedProposalsCount::mutate(|i| *i += 1);
				debug::info!("all_confirmed_proposals_count is {:?}", Self::all_confirmed_proposals_count());
			}
			
			Self::deposit_event(RawEvent::WeatherVoteSubmitted(sender, hash));
			Ok(())
		}

		pub fn submit_proposal_authority_reputation(origin, who: T::AccountId) -> dispatch::DispatchResult {

			let sender = ensure_signed(origin)?;

			if !Self::is_vote_authority(&sender) {
				// TODO: return error
				debug::info!("{:?} is not in the vote authority list.", &sender);
				return Ok(());
			}
	
			if !Self::is_proposal_authority(&who) {
				debug::info!("{:?} is not in the proposal authority list.", &who);
				return Ok(());
			}

			if <Self as Store>::ProposalAuthorityReputation::get(&who).iter().find(|&j| j == &sender).is_some() {
				debug::info!("{:?} already votes for this proposal.", &sender);
				return Ok(());
			}
			else {
				<Self as Store>::ProposalAuthorityReputation::mutate(&who, |i| {
					// if !i.iter().find(|&j| j == &sender).is_some() {
					i.push(sender.clone());
					debug::info!("ProposalAuthorityReputation new vote is {:?}", &sender);
					// }
				});
			}
	
			debug::info!("ProposalAuthorityReputation is {:?}", <Self as Store>::ProposalAuthorityReputation::get(&who));
			
			// FIXME: check if AllConfirmedProposalsIndex or hash is duplicated (done)

			if <Self as Store>::ProposalAuthorityReputation::get(&who).len() as u64 == Self::vote_threshold() {
				Self::remove_proposal_authority(&who);
				<Self as Store>::ProposalAuthorityReputation::remove(&who);
				debug::info!("ProposalAuthority {:?} is deleted", &who);
			}
			
			Self::deposit_event(RawEvent::ProposalAuthorityReputationVoteSubmitted(sender, who));
			Ok(())
		}

		pub fn submit_vote_authority_reputation(origin, who: T::AccountId) -> dispatch::DispatchResult {

			let sender = ensure_signed(origin)?;

			if !Self::is_vote_authority(&sender) {
				// TODO: return error
				debug::info!("{:?} is not in the vote authority list.", &sender);
				return Ok(());
			}
	
			if !Self::is_vote_authority(&who) {
				// Self::remove_proposal_authority(&who);
				debug::info!("{:?} is not in the proposal authority list.", &who);
				return Ok(());
			}

			if <Self as Store>::VoteAuthorityReputation::get(&who).iter().find(|&j| j == &sender).is_some() {
				debug::info!("{:?} already votes for this proposal.", &sender);
				return Ok(());
			}
			else {
				<Self as Store>::VoteAuthorityReputation::mutate(&who, |i| {
					// if !i.iter().find(|&j| j == &sender).is_some() {
					i.push(sender.clone());
					debug::info!("VoteAuthorityReputation new vote is {:?}", &sender);
					// }
				});
			}
	
			debug::info!("VoteAuthorityReputation is {:?}", <Self as Store>::VoteAuthorityReputation::get(&who));
			
			// FIXME: check if AllConfirmedProposalsIndex or hash is duplicated (done)

			if <Self as Store>::VoteAuthorityReputation::get(&who).len() as u64 == Self::vote_threshold() {
				Self::remove_vote_authority(&who);
				<Self as Store>::VoteAuthorityReputation::remove(&who);
				debug::info!("VoteAuthority {:?} is deleted", &who);
			}
			
			Self::deposit_event(RawEvent::VoteAuthorityReputationVoteSubmitted(sender, who));
			Ok(())
		}


		fn offchain_worker(block_number: T::BlockNumber) {
			// use support::debug::info;

			if true {
			debug::info!("Offchain worker starts!");
			debug::info!("Current block number is: {:?}", block_number);
			// debug::info!("Something is {:?}", Something::get());
			// Something::put(6);
			// debug::info!("Something is {:?}", Something::get());
			// info!("Parent block hash: {:?}", <system::Module<T>>::block_hash(block_number - 1.into()));
			let weather = match Self::fetch_weather() {
				Ok(weather) => {
				  debug::info!("Got weather: {:?}", weather);
				  weather
				},
				_ => {
				  debug::error!("Error fetching BTC price.");
				  return
				}
			};
			// Something::put(price as u32);
			// debug::info!("Something is {:?}", Something::get());
			// Nonce::put(247);
			
			// debug::info!("Nonce before mutate is {:?}", Nonce::get());
			Nonce::mutate(|n| {
				*n += 1
			});
			// debug::info!("Nonce after mutate is {:?}", Nonce::get());
			// // Nonce::put(43);
			let nonce = Nonce::get();
			debug::info!("Nonce is {:?}", &nonce);

			let random_num = random_seed();
			debug::info!("random_seed is {:?}", &random_num);
			let random_hash = (&random_num, &nonce)
				.using_encoded(T::Hashing::hash);
			
			// <AllProposalsIndex<T>>::insert(&random_hash, &nonce);

			debug::info!("random_hash: {:?}", &random_hash);

			// <Proposals<T>>::insert(&random_hash, &weather);
			debug::info!("Weather is {:?}", &weather);

			// <AllProposalsArray<T>>::insert(&nonce, &random_hash);
			// AllProposalsCount::put(&nonce);

			Self::submit_weather_proposal_on_chain(weather, random_hash);
		}
		}
	}
}

impl<T: Trait> Module<T> {

	/// Helper that confirms whether the given `AccountId` can send `proposal` transactions
	fn is_proposal_authority(who: &T::AccountId) -> bool {
		let result = Self::proposal_authorities().into_iter().find(|i| i == who).is_some();
		debug::info!("is_proposal_authority result is {:?}", &result);
		result
	}

	/// TODO: find Query's usage
	fn remove_proposal_authority(who: &T::AccountId){
		<Self as Store>::ProposalAuthorities::mutate(|l| {
			let position = Self::proposal_authorities().into_iter().position(|i| i == *who).unwrap();
			debug::info!("remove_proposal_authority position is {:?}", &position);
			l.remove(position);
		});
	}

	/// Helper that confirms whether the given `AccountId` can send `proposal` transactions
	fn is_vote_authority(who: &T::AccountId) -> bool {
		let result = Self::vote_authorities().into_iter().find(|i| i == who).is_some();
		debug::info!("is_vote_authority result is {:?}", &result);
		result
	}

	fn remove_vote_authority(who: &T::AccountId){
		<Self as Store>::VoteAuthorities::mutate(|l| {
			let position = Self::vote_authorities().into_iter().position(|i| i == *who).unwrap();
			debug::info!("remove_vote_authority position is {:?}", &position);
			l.remove(position);
		});
	}

	fn parse_weather_json(json_val: JsonValue) -> Result<Weather, http::Error> {
		let main = json_val.get_object()[1].1.get_array()[0].get_object()[1].1.get_string();
		let description = json_val.get_object()[1].1.get_array()[0].get_object()[2].1.get_string();
		let icon = json_val.get_object()[1].1.get_array()[0].get_object()[3].1.get_string();
		debug::warn!("main, description, icon is {:?}, {:?}, {:?}", &main, &description, &icon);

		let temp_f64 = json_val.get_object()[3].1.get_object()[0].1.get_number_f64();
		let temp = (temp_f64 * 1000.0).round() as u32;
		let humidity_f64 = json_val.get_object()[3].1.get_object()[5].1.get_number_f64();
		let humidity = (humidity_f64 * 1000.0).round() as u32;
		debug::warn!("temp, humidity is {:?}, {:?}", &temp, &humidity);

		let wind_f64 = json_val.get_object()[5].1.get_object()[0].1.get_number_f64();
		let wind = (wind_f64 * 1000.0).round() as u32;
		let clouds_f64 = json_val.get_object()[6].1.get_object()[0].1.get_number_f64();
		let clouds = (clouds_f64 * 1000.0).round() as u32;

		let time = json_val.get_object()[7].1.get_number_f64() as u64;

		let sunrise = json_val.get_object()[8].1.get_object()[3].1.get_number_f64() as u64;
		let sunset = json_val.get_object()[8].1.get_object()[4].1.get_number_f64() as u64;

		let city = json_val.get_object()[11].1.get_string();

		debug::warn!("time: {:?} \n 
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
		debug::warn!("main_bytes, description_bytes, icon_bytes, city_bytes is {:?}, {:?}, {:?}, {:?}", &main_bytes, &description_bytes, 
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
		debug::warn!("weather is {:?}", &weather);
		Ok(weather)

	}
	
	fn fetch_weather() -> Result<Weather, http::Error> {
		let pending = http::Request::get(
        "https://api.openweathermap.org/data/2.5/weather?q=London,uk&APPID=47c69406d636336123cbef9721328177"
      ).send().map_err(|_| http::Error::IoError)?;

		let response = pending.wait()?;
		if response.code != 200 {
			debug::warn!("Unexpected status code: {}", response.code);
			return Err(http::Error::Unknown);
		}

		//   let body = response.body().collect::<Vec<u8>>();
		// const START_IDX: usize = "{\"USD\":".len();
		let body = response.body().collect::<Vec<u8>>();
		debug::info!("Weather json in");
		debug::warn!("Body: {:?}", core::str::from_utf8(&body).ok());
		let json_val: JsonValue = simple_json::parse_json(
			&core::str::from_utf8(&body).unwrap())
			.unwrap();

		let weather = Self::parse_weather_json(json_val).unwrap();

		Ok(weather)
	}


	fn submit_weather_proposal_on_chain(weather: Weather, hash: T::Hash) {
		use system::offchain::SubmitSignedTransaction;
		// let call = Call::submit_weather_proposal(weather);
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
		// Moment = <T as timestamp::Trait>::Moment,
	{
		// To emit this event, we call the deposit funtion, from our runtime funtions
		// SomethingStored(u32, AccountId),
		ProposalAuthoritiesAdded(AccountId),
		ProposalAuthoritiesDeleted(AccountId),
		VoteAuthoritiesAdded(AccountId),
		VoteAuthoritiesDeleted(AccountId),
		VoteThresholdSet(u64),
		WeatherProposalSubmitted(AccountId, Hash, Weather),
		WeatherVoteSubmitted(AccountId, Hash),
		ProposalAuthorityReputationVoteSubmitted(AccountId, AccountId),
		VoteAuthorityReputationVoteSubmitted(AccountId, AccountId),
	}
);

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
