<template>
  <div class="app-names">
    <PageHeader
      title="Name Auctions"
      :has-crumbs="true"
      :page="{to: '/auctions', name: 'Name Auctions'}"
    />
    <div v-if="!loading && auctions.length > 0">
      <NameAuctionList>
        <NameAuction
          v-for="(item, index) of auctions"
          :key="index"
          :data="item"
        />
      </NameAuctionList>
      <LoadMoreButton @update="loadMore" />
    </div>
    <div v-if="loading">
      Loading....
    </div>
    <div v-if="!loading && auctions.length == 0">
      Nothing to see here right now....
    </div>
  </div>
</template>

<script>
import NameAuctionList from '../../partials/names/nameAuctionList'
import NameAuction from '../../partials/names/nameAuction'
import PageHeader from '../../components/PageHeader'
import LoadMoreButton from '../../components/loadMoreButton'

export default {
  name: 'AppNames',
  components: {
    NameAuctionList,
    NameAuction,
    PageHeader,
    LoadMoreButton
  },
  data () {
    return {
      page: 1,
      loading: true,
      auctions: []
    }
  },
  async asyncData ({ store }) {
    const auctions = await store.dispatch('names/getActiveNameAuctions', { 'page': 1, 'limit': 10 })
    for (const x of auctions) {
      console.log(x)
    }
    return { auctions, page: 2, loading: false }
  },
  methods: {
    async loadMore () {
      const auctions = await this.$store.dispatch('names/getActiveNameAuctions', { 'page': this.page, 'limit': 10 })
      this.auctions = [...this.auctions, ...auctions]
      this.page += 1
    }
  }
}
</script>
