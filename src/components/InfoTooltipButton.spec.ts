import { mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';
import InfoTooltipButton from './InfoTooltipButton.vue';

describe('InfoTooltipButton.vue', () => {
  it('is closed by default', () => {
    const wrapper = mount(InfoTooltipButton, { props: { text: 'explanation' } });
    expect(wrapper.find('[role="tooltip"]').exists()).toBe(false);
  });

  it('opens on click', async () => {
    const wrapper = mount(InfoTooltipButton, { props: { text: 'explanation' } });
    await wrapper.find('button').trigger('click');
    expect(wrapper.find('[role="tooltip"]').text()).toBe('explanation');
    expect(wrapper.find('button').attributes('aria-expanded')).toBe('true');
  });

  it('closes on a second click', async () => {
    const wrapper = mount(InfoTooltipButton, { props: { text: 'explanation' } });
    await wrapper.find('button').trigger('click');
    await wrapper.find('button').trigger('click');
    expect(wrapper.find('[role="tooltip"]').exists()).toBe(false);
  });

  it('closes on Escape and returns focus to the trigger button', async () => {
    const wrapper = mount(InfoTooltipButton, { props: { text: 'explanation' }, attachTo: document.body });
    await wrapper.find('button').trigger('click');
    expect(wrapper.find('[role="tooltip"]').exists()).toBe(true);

    await wrapper.find('span').trigger('keydown', { key: 'Escape' });

    expect(wrapper.find('[role="tooltip"]').exists()).toBe(false);
    expect(document.activeElement).toBe(wrapper.find('button').element);
  });

  it('has an accessible name', () => {
    const wrapper = mount(InfoTooltipButton, { props: { text: 'explanation' } });
    expect(wrapper.find('button').attributes('aria-label')).toBe('More info');
  });
});
