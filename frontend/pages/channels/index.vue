<template>
  <div class="app-names">
    <PageHeader title="State Channels">
      <BreadCrumbs />
    </PageHeader>
    <ChannelList>
      <nuxt-link
        v-for="(item, index) in channels"
        :key="index"
        :to="`/channels/transactions/${item}`"
      >
        <Channel
          :data="item"
        />
      </nuxt-link>
    </ChannelList>
  </div>
</template>

<script>
import ChannelList from '../../partials/channels/channelList'
import Channel from '../../partials/channels/channel'
import PageHeader from '../../components/PageHeader'
import BreadCrumbs from '../../components/breadCrumbs'
import { mapState } from 'vuex'

export default {
  name: 'AppChannels',
  components: {
    ChannelList,
    Channel,
    PageHeader,
    BreadCrumbs
  },
  data () {
    return {
      page: 1
    }
  },
  computed: {
    ...mapState('channels', [
      'channels'
    ])
  },
  async beforeMount () {
    await this.$store.dispatch('channels/getChannels')
  }
}
</script>
