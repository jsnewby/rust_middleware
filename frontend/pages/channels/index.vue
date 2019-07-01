<template>
  <div class="app-names">
    <PageHeader
      :has-crumbs="true"
      :page="{to: '/channels', name: 'Channels'}"
      title="State Channels"
    />
    <ChannelList>
      <Channel
        v-for="(item, index) in channels"
        :key="index"
        :data="item"
      />
    </ChannelList>
  </div>
</template>

<script>
import ChannelList from '../../partials/channels/channelList'
import Channel from '../../partials/channels/channel'
import PageHeader from '../../components/PageHeader'
import { mapState } from 'vuex'

export default {
  name: 'AppChannels',
  components: {
    ChannelList,
    Channel,
    PageHeader
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
