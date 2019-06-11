<template>
  <div class="app-contracts">
    <PageHeader title="Contracts">
      <BreadCrumbs />
    </PageHeader>
    <ContractList>
      <nuxt-link
        v-for="(item, index) in Object.values(contracts)"
        :key="index"
        :to="`/contracts/transactions/${item.contract_id}`"
      >
        <Contract
          :data="item"
        />
      </nuxt-link>
    </ContractList>
    <LoadMoreButton @update="loadMore" />
  </div>
</template>

<script>

import ContractList from '../../partials/contractList'
import Contract from '../../partials/contract'
import PageHeader from '../../components/PageHeader'
import BreadCrumbs from '../../components/breadCrumbs'
import LoadMoreButton from '../../components/loadMoreButton'
import { mapState } from 'vuex'

export default {
  name: 'AppContracts',
  components: {
    ContractList,
    Contract,
    PageHeader,
    BreadCrumbs,
    LoadMoreButton
  },
  data () {
    return {
      page: 1
    }
  },
  computed: {
    ...mapState('contracts', [
      'contracts'
    ])
  },
  beforeMount () {
    this.loadMore()
  },
  methods: {
    loadMore () {
      this.$store.dispatch('contracts/getContracts', { 'page': this.page, 'limit': 10 })
      this.page += 1
    }
  }
}
</script>
