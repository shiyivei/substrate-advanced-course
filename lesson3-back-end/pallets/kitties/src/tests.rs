use super::*;

use frame_support::{assert_noop, assert_ok};
use mock::{new_test_ext, Event as TestEvent, KittiesModule, Origin, System, Test};
use std::ops::Add; //这里引入标准库中的方法进行计算

//构建三个address,开发链有一系列测试账户，其中前两个账户有一定数额的测试余额
const ALICE: u64 = 0; //100
const BOB: u64 = 1; //25
const CHARLIE: u64 = 2; //1

#[test]
fn it_should_work_for_create_kitty() {
	new_test_ext().execute_with(|| {
		let alice: u64 = ALICE;
		//获取当前kitty的id,创建时会默认将该id给新的kitty
		let kitty_id = NextKittyId::<Test>::get();

		//断言是否创建成功
		assert_ok!(KittiesModule::create(Origin::signed(alice)));

		//断言next_kitty_id是否可以成功增加
		assert_eq!(KittiesModule::next_kitty_id(), kitty_id.add(&1));

		//断言新创建的kitty持有人是当前创建者
		assert_eq!(KittyOwner::<Test>::get(kitty_id), Some(alice));

		//断言kitty已经存储,assert_ne 和 None 双重否定（即断言成功），为什么要使用这个，因为我们无法判断生成的kitty的具体值，但是可以确定它是有值还是无值，所以就可以使用这种方式来对存在性进行判断
		assert_ne!(Kitties::<Test>::get(kitty_id), None);

		//断言下一个kitty的id
		assert_eq!(NextKittyId::<Test>::get(), kitty_id.add(&1));

		//测试event
		let kitty = Kitties::<Test>::get(kitty_id).unwrap();
		System::assert_has_event(TestEvent::KittiesModule(Event::KittyCreated(
			alice, kitty_id, kitty,
		)));
	});
}

#[test]
fn create_kitty_should_failed_when_not_enough_balance() {
	new_test_ext().execute_with(|| {
		//charlie 账户没有钱,所以无法创建
		let charlie: u64 = CHARLIE;

		assert_noop!(
			KittiesModule::create(Origin::signed(charlie)),
			Error::<Test>::NotEnoughBalance
		);
	});
}

#[test]
fn create_kitty_should_failed_when_kitty_id_is_invalid() {
	new_test_ext().execute_with(|| {
		//charlie 账户没有钱,所以无法创建
		let bob: u64 = BOB;

		//因为KittyIndex可以在外部设置，所以我们手动设置一个无效值
		let max_index = <Test as Config>::KittyIndex::max_value();
		NextKittyId::<Test>::set(max_index);

		
		assert_noop!(KittiesModule::create(Origin::signed(bob)), Error::<Test>::InvalidKittyId);
	});
}

#[test]
fn create_kitty_should_failed_when_own_too_many_kitties() {
	new_test_ext().execute_with(|| {
		let alice: u64 = ALICE;
		//因为KittyIndex可以在外部设置，所以我们手动设置一个无效值,我么之前设置了最多能有三个kitties
		//type MaxKittyIndex = ConstU32<3>;
		//我们连续创建3个
		assert_ok!(KittiesModule::create(Origin::signed(alice)));
		assert_ok!(KittiesModule::create(Origin::signed(alice)));
		assert_ok!(KittiesModule::create(Origin::signed(alice)));

		// 然后断言创建第4个
		assert_noop!(KittiesModule::create(Origin::signed(alice)), Error::<Test>::OwnTooManyKitties);
	});
}

#[test]

fn it_should_work_for_breed_kitty() {
	new_test_ext().execute_with(|| {
		let alice: u64 = ALICE;
		let kitty_id_1 = NextKittyId::<Test>::get();
		assert_ok!(KittiesModule::create(Origin::signed(alice)));

		let kitty_id_2 = NextKittyId::<Test>::get();
		assert_ok!(KittiesModule::create(Origin::signed(alice)));

		let new_kitty_id = NextKittyId::<Test>::get();

		//断言是否创建成功
		assert_ok!(KittiesModule::breed(Origin::signed(alice), kitty_id_1, kitty_id_2));

		//断言新创建的kitty持有人是当前创建者
		assert_eq!(KittyOwner::<Test>::get(new_kitty_id), Some(alice));

		//断言kitty已经存储,assert_ne 和 None 双重否定（即断言成功），为什么要使用这个，因为我们无法判断生成的kitty的具体值，但是可以确定它是有值还是无值，所以就可以使用这种方式来对存在性进行判断
		assert_ne!(Kitties::<Test>::get(new_kitty_id), None);

		//断言下一个kitty的id
		assert_eq!(NextKittyId::<Test>::get(), new_kitty_id.add(&1));

		//断言被资金被锁定
		assert_eq!(
			<Test as Config>::Currency::reserved_balance(&alice),
			<Test as Config>::KittyPrice::get().checked_mul(3).unwrap()
		);

		//测试event
		let kitty = Kitties::<Test>::get(new_kitty_id).unwrap();
		System::assert_has_event(TestEvent::KittiesModule(Event::KittyBred(
			alice,
			new_kitty_id,
			kitty,
		)));
	})
}

#[test]
fn it_should_fail_when_not_enough_balance() {
	new_test_ext().execute_with(|| {
		let bob: u64 = BOB;
		let kitty_id_1 = NextKittyId::<Test>::get();
		assert_ok!(KittiesModule::create(Origin::signed(bob)));

		let kitty_id_2 = NextKittyId::<Test>::get();
		assert_ok!(KittiesModule::create(Origin::signed(bob)));

		assert_noop!(
			KittiesModule::breed(Origin::signed(bob), kitty_id_1, kitty_id_2),
			Error::<Test>::NotEnoughBalance
		);
	})
}

#[test]

fn breed_kitty_should_failed_when_kitty_id_is_same() {
	new_test_ext().execute_with(|| {
		let alice: u64 = ALICE;
		let kitty_id_1 = NextKittyId::<Test>::get();
		assert_ok!(KittiesModule::create(Origin::signed(alice)));

		let _kitty_id_2 = NextKittyId::<Test>::get();
		assert_ok!(KittiesModule::create(Origin::signed(alice)));

		//因为KittyIndex可以在外部设置，所以我们手动设置一个无效值
		// let max_index = <Test as Config>::KittyIndex::max_value();
		// NextKittyId::<Test>::set(max_index);

		assert_noop!(
			KittiesModule::breed(Origin::signed(alice), kitty_id_1, kitty_id_1),
			Error::<Test>::SameKittyId
		);
	});
}

#[test]
fn breed_kitty_should_failed_when_kitty_id_is_invalid() {
	new_test_ext().execute_with(|| {
		let alice: u64 = ALICE;
		let kitty_id_1 = NextKittyId::<Test>::get();
		assert_ok!(KittiesModule::create(Origin::signed(alice)));

		let _kitty_id_2 = NextKittyId::<Test>::get();
		assert_ok!(KittiesModule::create(Origin::signed(alice)));

		let kitty_id_3 = NextKittyId::<Test>::get();
		

		assert_noop!(
			KittiesModule::breed(Origin::signed(alice), kitty_id_1, kitty_id_3),
			Error::<Test>::InvalidKittyId
		);
	});
}


#[test]
fn breed_kitty_should_failed_when_own_too_many_kitties() {
	new_test_ext().execute_with(|| {
		let alice: u64 = ALICE;
		let kitty_id_1 = NextKittyId::<Test>::get();
		assert_ok!(KittiesModule::create(Origin::signed(alice)));

		let kitty_id_2 = NextKittyId::<Test>::get();
		assert_ok!(KittiesModule::create(Origin::signed(alice)));

		//再创建一只
		assert_ok!(KittiesModule::create(Origin::signed(alice)));


		//断言是否创建成功
		// assert_ok!(KittiesModule::breed(Origin::signed(alice), kitty_id_1, kitty_id_2));

		assert_noop!(
			KittiesModule::breed(Origin::signed(alice), kitty_id_1, kitty_id_2),
			Error::<Test>::OwnTooManyKitties
		);
	});
}


#[test]
fn it_should_work_for_transfer_kitty() {
	new_test_ext().execute_with(|| {
		let alice: u64 = ALICE;
		let bob: u64 = BOB;

		//获取当前kitty的id,创建时会默认将该id给新的kitty
		let kitty_id = NextKittyId::<Test>::get();

		//断言是否创建成功
		assert_ok!(KittiesModule::create(Origin::signed(alice)));
		assert_ok!(KittiesModule::transfer(Origin::signed(alice), kitty_id,bob));

		//断言next_kitty_id是否可以成功增加
		assert_eq!(KittiesModule::next_kitty_id(), kitty_id.add(&1));

		//断言新创建的kitty持有人是当前创建者
		assert_eq!(KittyOwner::<Test>::get(kitty_id), Some(bob));

		//断言kitty已经存储,assert_ne 和 None 双重否定（即断言成功），为什么要使用这个，因为我们无法判断生成的kitty的具体值，但是可以确定它是有值还是无值，所以就可以使用这种方式来对存在性进行判断
		assert_ne!(Kitties::<Test>::get(kitty_id), None);

		//断言下一个kitty的id
		assert_eq!(NextKittyId::<Test>::get(), kitty_id.add(&1));

		//断言被资金被锁定
		assert_eq!(
			<Test as Config>::Currency::reserved_balance(&alice),0
		);
		assert_eq!(
			<Test as Config>::Currency::reserved_balance(&bob),<Test as Config>::KittyPrice::get()
		);

		//测试event
		System::assert_has_event(TestEvent::KittiesModule(Event::KittyTransferred(
			alice, bob, kitty_id
		)));
	});
}

#[test]
fn transfer_kitty_fail_when_not_enough_balance() {
	new_test_ext().execute_with(|| {
		let alice: u64 = ALICE;
		let bob: u64 = BOB;

		//创建两个
		assert_ok!(KittiesModule::create(Origin::signed(bob)));

		//获取当前kitty的id,创建时会默认将该id给新的kitty
		let kitty_id = NextKittyId::<Test>::get();

		assert_ok!(KittiesModule::create(Origin::signed(bob)));

		assert_noop!(KittiesModule::transfer(Origin::signed(bob),kitty_id,alice),Error::<Test>::NotEnoughBalance);
		
	});
}

#[test]

fn transfer_kitty_fail_when_own_too_many_kitties() {
	new_test_ext().execute_with(|| {
		let alice: u64 = ALICE;
		let bob: u64 = BOB;

		//断言是否创建成功
		assert_ok!(KittiesModule::create(Origin::signed(alice)));
		assert_ok!(KittiesModule::create(Origin::signed(alice)));
		assert_ok!(KittiesModule::create(Origin::signed(alice)));
		// assert_ok!(KittiesModule::create(Origin::signed(alice)));

		//获取当前kitty的id,创建时会默认将该id给新的kitty
		let kitty_id = NextKittyId::<Test>::get();
		assert_ok!(KittiesModule::create(Origin::signed(bob)));

		// print!("before")

		assert_noop!(KittiesModule::transfer(Origin::signed(bob),kitty_id,alice),Error::<Test>::OwnTooManyKitties);
	});
}

#[test]
fn transfer_kitty_fail_when_not_owner() {
	new_test_ext().execute_with(|| {
		let alice: u64 = ALICE;
		let bob: u64 = BOB;

		//获取当前kitty的id,创建时会默认将该id给新的kitty
		let kitty_id = NextKittyId::<Test>::get();

		//断言是否创建成功
		assert_ok!(KittiesModule::create(Origin::signed(alice)));

		assert_noop!(KittiesModule::transfer(Origin::signed(bob),kitty_id,alice),Error::<Test>::NotOwner);
		
	});
}