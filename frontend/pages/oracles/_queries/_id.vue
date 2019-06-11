<template>
  <div class="app-transactions">
    <PageHeader title="Oracle Queries">
      <BreadCrumbs />
    </PageHeader>
    <div
      v-if="queries.length > 0"
    >
      <OracleList>
        <OracleQuery
          v-for="(item, index) of queries"
          :key="index"
          :data="item"
        />
      </OracleList>
      <LoadMoreButton @update="loadMore" />
    </div>
    <div v-else>
      Nothing to see here right now....
    </div>
  </div>
</template>

<script>

import OracleList from '../../../partials/oracles/oracleList'
import OracleQuery from '../../../partials/oracles/oracleQuery'
import PageHeader from '../../../components/PageHeader'
import BreadCrumbs from '../../../components/breadCrumbs'
import LoadMoreButton from '../../../components/loadMoreButton'

export default {
  name: 'OracleQueryResponse',
  components: {
    OracleList,
    OracleQuery,
    PageHeader,
    BreadCrumbs,
    LoadMoreButton
  },
  data () {
    return {
      oracleId: null,
      queries: [],
      page: 1
    }
  },
  async asyncData ({ store, params }) {
    const queries = await store.dispatch('oracles/getAllQueries', { oracleId: params.id, 'page': 1, 'limit': 10 })
    return { oracleId: params.id, queries, page: 2 }
  },
  methods: {
    async loadMore () {
      const queries = await this.$store.dispatch('oracles/getAllQueries', { oracleId: this.oracleId, 'page': this.page, 'limit': 10 })
      this.queries = [...this.queries, ...queries]
      this.page += 1
    }
  }
}
</script>
