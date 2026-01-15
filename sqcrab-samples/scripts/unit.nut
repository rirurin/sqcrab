function replenish_unit() {
    local current_hp = this.unit_get_hp();
    this.unit_set_hp(current_hp + 25);
    local current_mp = this.unit_get_mp();
    this.unit_set_mp(current_mp + 10);
}