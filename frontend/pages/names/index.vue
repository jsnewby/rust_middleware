<template>
  <div class="app-names">
    <PageHeader
      title="Names"
      :has-crumbs="true"
      :page="{to: '/names', name: 'Names'}"
    />
    <div v-if="Object.values(names).length">
      <NameList>
        <Name
          v-for="(item, index) in Object.values(names)"
          :key="index"
          :data="item"
        />
      </NameList>
      <LoadMoreButton @update="loadMore" />
    </div>
    <div v-else>
      Nothing to see here right now....
    </div>
  </div>
</template>

<script>
import NameList from '../../partials/names/nameList'
import Name from '../../partials/names/name'
import PageHeader from '../../components/PageHeader'
import LoadMoreButton from '../../components/loadMoreButton'
import { mapState } from 'vuex'

export default {
  name: 'AppNames',
  components: {
    NameList,
    Name,
    PageHeader,
    LoadMoreButton
  },
  data () {
    return {
      page: 1
    }
  },
  computed: {
    ...mapState('names', [
      'names'
    ])
  },
  beforeMount () {
    this.loadMore()
  },
  methods: {
    loadMore () {
      this.$store.dispatch('names/getNames', { 'page': this.page, 'limit': 10 })
      this.page += 1
    }
  }
}
</script>
