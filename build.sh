#!/usr/bin/env bash

VERSION=0.1
WORK_DIR=$(cd $(dirname $0); pwd)
BASE_MODULE=dao_base
CATEGORY_MODULE=dao_category
DAOFACTORY_MODULE=dao_factory
MANAGER_MODULE=dao_manager
PROPOSAL_MODULE=dao_proposal
SETTING_MODULE=dao_setting
USERS_MODULE=dao_users
VAULT_MODULE=dao_vault
ERC20_MODULE=erc20
TEMPLATE_MODULE=template_manager


function build_module() {
    m_name=$1
    m_dir=${WORK_DIR}/${m_name}
    echo "build module ${m_dir}"
    cd ${m_dir}
    cargo +nightly contract build
    if [ $? -ne 0 ];then
      echo "build module failed"
      exit 1
    fi
    echo "copy to ../release"
    cp ${m_dir}/target/ink/${m_name}.wasm ../release/${m_name}_v$VERSION.wasm
    cp ${m_dir}/target/ink/${m_name}.contract ../release/${m_name}_v$VERSION.contract
    cp ${m_dir}/target/ink/metadata.json ../release/${m_name}_v$VERSION.json
    cd -
}

echo "clean release"
rm -rf ${WORK_DIR}/release
mkdir -p ${WORK_DIR}/release

build_module ${BASE_MODULE}
build_module ${CATEGORY_MODULE}
build_module ${DAOFACTORY_MODULE}
build_module ${MANAGER_MODULE}
build_module ${SETTING_MODULE}
build_module ${PROPOSAL_MODULE}
build_module ${USERS_MODULE}
build_module ${VAULT_MODULE}
build_module ${ERC20_MODULE}
build_module ${TEMPLATE_MODULE}
