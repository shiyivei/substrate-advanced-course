use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok, BoundedVec};

//1.测试创建存证函数
//1.1 通过测试
//1.2 失败测试

#[test]
fn create_claim_works() {
	new_test_ext().execute_with(|| {
		//定义测试输入
		let claim = vec![1, 2];
		//断言测试能够成功执行
		assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));

		//转换输入
		let bounded_claim =
			BoundedVec::<u8, <Test as Config>::MaxClaimLength>::try_from(claim.clone()).unwrap();
		//断言存储结果
		assert_eq!(
			Proofs::<Test>::get(&bounded_claim),
			Some((1, frame_system::Pallet::<Test>::block_number()))
		);
	})
}

#[test]
fn create_claim_failed_when_claim_already_exist() {
	new_test_ext().execute_with(|| {
		//构造输入
		let claim: Vec<u8> = vec![0, 1];
		//执行存储
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());
		//判断存储已经存在
		assert_noop!(
			PoeModule::create_claim(Origin::signed(1), claim.clone()),
			Error::<Test>::ProofAlreadyExist
		);
	})
}

#[test]
fn create_claim_failed_when_claim_too_long() {
	new_test_ext().execute_with(|| {
		//构造输入
		let claim = vec![0, 255];
		//
		assert_noop!(
			PoeModule::create_claim(Origin::signed(1), claim.clone()),
			Error::<Test>::ClaimTooLong
		);
	})
}

#[test]
fn revoke_claim_failed_when_claim_not_exist() {
	new_test_ext().execute_with(|| {
		//构造输入参数
		let claim: Vec<u8> = vec![0, 1];

		//不执行存储

		//查看是否能删除
		assert_noop!(
			PoeModule::revoke_claim(Origin::signed(1), claim.clone()),
			Error::<Test>::ClaimNotExist
		);
	})
}

#[test]
fn transfer_claim_failed_when_claim_not_exist() {
	new_test_ext().execute_with(|| {
		//构造输入参数
		let claim: Vec<u8> = vec![0, 1];
		let dest: u64 = 2;

		//创建存储
		assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));
		//转移存储
		assert_ok!(PoeModule::transfer_claim(Origin::signed(1), claim.clone(), dest));

		//转换输入
		let bounded_claim =
			BoundedVec::<u8, <Test as Config>::MaxClaimLength>::try_from(claim.clone()).unwrap();

		//查看存储结果，是否被转移
		assert_eq!(
			Proofs::<Test>::get(&bounded_claim),
			Some((2, frame_system::Pallet::<Test>::block_number()))
		);
	})
}

#[test]
fn transfer_claim_works() {
	new_test_ext().execute_with(|| {
		//构造输入参数
		let claim: Vec<u8> = vec![0, 1];
		let dest: u64 = 2;

		//不执行任何操作

		//查看转移报错
		assert_noop!(
			PoeModule::transfer_claim(Origin::signed(1), claim.clone(), dest),
			Error::<Test>::ClaimNotExist
		);
	})
}
