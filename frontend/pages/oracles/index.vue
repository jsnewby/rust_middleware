<template>
  <div class="app-oracles">
    <PageHeader title="Oracles">
      <BreadCrumbs />
    </PageHeader>
    <OracleList>
      <Oracle
        v-for="(item, index) of Object.values(oracles)"
        :key="index"
        :data="item"
      />
    </OracleList>
    <LoadMoreButton @update="loadMore" />
  </div>
</template>

<script>
import OracleList from '../../partials/oracles/oracleList'
import Oracle from '../../partials/oracles/oracle'
import PageHeader from '../../components/PageHeader'
import BreadCrumbs from '../../components/breadCrumbs'
import LoadMoreButton from '../../components/loadMoreButton'
import { mapState } from 'vuex'

export default {
  name: 'AppOracles',
  components: {
    OracleList,
    Oracle,
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
    ...mapState('oracles', [
      'oracles'
    ])
  },
  beforeMount () {
    this.loadMore()
  },
  methods: {
    loadMore () {
      this.$store.dispatch('oracles/getOracles', { 'page': this.page, 'limit': 10 })
      this.page += 1
    }
  }
}
</script>
