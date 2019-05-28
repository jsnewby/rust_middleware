<template>
  <div class="app-contracts">
    <PageHeader title="Contracts">
      <BreadCrumbs />
    </PageHeader>
    <ContractList>
      <Contract
        v-for="(item, index) in contracts"
        :key="index"
        :data="item"
      />
    </ContractList>
  </div>
</template>

<script>

import ContractList from '../../partials/contractList'
import Contract from '../../partials/contract'
import PageHeader from '../../components/PageHeader'
import BreadCrumbs from '../../components/breadCrumbs'

import { mapState } from 'vuex'

export default {
  name: 'AppContracts',
  components: {
    ContractList,
    Contract,
    PageHeader,
    BreadCrumbs
  },
  computed: {
    ...mapState('contracts', [
      'contracts'
    ])
  },
  async beforeMount () {
    await this.$store.dispatch('contracts/getAllContracts')
  }
}
</script>
